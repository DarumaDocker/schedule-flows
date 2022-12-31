import { NextRequest, NextResponse } from 'next/server';
import { redis } from '@/lib/upstash';

export default async (req: NextRequest) => {
    const flowsUser = req.nextUrl.searchParams.get('flows_user');
    const flowId = req.nextUrl.searchParams.get('flow_id');
    const cronStr = req.nextUrl.searchParams.get('cron');
    const body = await req.text();
  
    if (!flowsUser || !flowId) {
        return new NextResponse('Bad request', {status: 400});
    }
  
    try {
        let scheduleId;
        let lKey;
        let cron: any = await redis.get(`${flowId}:cron`);
        if (cron) {
            scheduleId = cron.schedule_id;
            let res = await fetch(`https://qstash.upstash.io/v1/schedules/${scheduleId}`, {
                method: 'DELETE'
            });

            if (!res || !res.ok) {
                throw await res.text();
            }

            lKey = cron.l_key;
            await redis.del(`${lKey}:scheduler`);
            await redis.del(`${flowId}:cron`);
        }

        lKey = makeKey(10);

        let res = await fetch(`https://qstash.upstash.io/v1/publish/${process.env.SCHEDULE_HOOK_URL_PREFIX}?l_key=${lKey}`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${process.env.UPSTASH_QSTASH_TOKEN}`,
                'Upstash-Cron': `0 0 ${cronStr}`
            },
            body
        });

        if (!res.ok) {
            throw await res.text();
        }

        let result = await res.json();
        scheduleId = result.scheduleId;

        await redis.set(`${lKey}:scheduler`, {
          flow_id: flowId,
          flows_user: flowsUser,
          schedule_id: scheduleId
        });

        let r = {
          l_key: lKey,
          flows_user: flowsUser,
          schedule_id: scheduleId
        };
        await redis.set(`${flowId}:cron`, r);

        return NextResponse.json(r);
    } catch(e: any) {
        return new NextResponse(e.toString(), {status: 500});
    }
};

function makeKey(length: number) {
    var result           = '';
    var characters       = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    var charactersLength = characters.length;
    for ( var i = 0; i < length; i++ ) {
        result += characters.charAt(Math.floor(Math.random() * charactersLength));
    }
    return result;
}

export const config = {
  runtime: 'experimental-edge',
};



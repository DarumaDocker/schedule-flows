import { NextRequest, NextResponse } from 'next/server';
import { redis } from '@/lib/upstash';

export default async (req: NextRequest) => {
    const lKey = req.nextUrl.searchParams.get('l_key');
  
    if (!lKey) {
        return new NextResponse('Bad request', {status: 400});
    }
  
    try {
        let scheduler = await redis.get(`${lKey}:scheduler`);

        if (scheduler) {
          return NextResponse.json(scheduler);
        } else {
          return new NextResponse('No flow binding with the key', {status: 404});
        }
    } catch(e: any) {
        return new NextResponse(e.toString(), {status: 500});
    }
};

export const config = {
  runtime: 'experimental-edge',
};

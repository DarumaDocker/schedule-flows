This is a library for scheduling your flow function as a cron job in [flows.network](https://flows.network).

## Usage example
```rust
use schedule_flows::{schedule_cron_job, schedule_handler};
use slack_flows::send_message_to_channel;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    schedule_cron_job(String::from("50 8 * * *"), String::from("cron_job_evoked")).await;
}

#[schedule_handler]
async fn handler(body: Vec<u8>) {
    send_message_to_channel(
        "myworkspace",
        "mychannel",
        String::from_utf8_lossy(&body).into_owned(),
    ).await;
}
```

In `on_deploy()` the [`schedule_cron_job`](https://docs.rs/schedule-flows/latest/schedule_flows/fn.schedule_cron_job.html) will create a cron job that will run daily at 8:50 UTC.

When the cron job is evoked, the `handler()` will be called and we send body to the [Slack](https://docs.rs/slack-flows).

The whole document is [here](https://docs.rs/schedule-flows).

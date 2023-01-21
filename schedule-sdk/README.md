This is a library for scheduling your flow function as a cron job in [test.flows.network](https://test.flows.network).

## Usage example
```rust
use schedule_flows::cron_job_evoked;
use slack_flows::send_message_to_channel;

#[no_mangle]
pub fn run() {
    if let Some(body) = cron_job_evoked(String::from("50 8 * * *"), String::from("cron_job_evoked"))
    {
        send_message_to_channel(
            "myworkspace",
            "mychannel",
            String::from_utf8_lossy(&body).into_owned(),
        );
    }
}
```

In `run()` the [`cron_job_evoked`](https://docs.rs/schedule-flows/latest/schedule_flows/fn.cron_job_evoked.html) will create a cron job that will run daily at 8:50 UTC.

When the cron job is evoked, the `run()` will be called again. We get the body then send it to the [Slack](https://docs.rs/slack-flows).

The whole document is [here](https://docs.rs/schedule-flows).

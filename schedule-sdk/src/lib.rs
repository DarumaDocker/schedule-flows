//! Make flow function acting as scheduled task in [Flows.network](https://test.flows.network)
//!
//! # Quick Start
//!
//! To get started, let's write a task that will send a message to Slack at a fixed time wach day.
//!
//! ```
//! use schedule_flows::schedule_cron_job;
//! use slack_flows::send_message_to_channel;
//!
//! #[no_mangle]
//! pub fn run() {
//!     schedule_cron_job(String::from("50 8 * * *"), String::from("cron_job_evoked"), |body| {
//!         send_message_to_channel(
//!             "myworkspace",
//!             "mychannel",
//!             String::from_utf8_lossy(&body).into_owned(),
//!         );
//!     });
//! }
//! ```
//!
//! [schedule_cron_job()] will create a cron job. The callback closure
//! will be called when the job is evoked at 8:50 UTC each day.
use http_req::request;
use lazy_static::lazy_static;

lazy_static! {
    static ref SCHEDULE_API_PREFIX: String = String::from(
        std::option_env!("SCHEDULE_API_PREFIX")
            .unwrap_or("https://schedule-flows-extension.vercel.app/api")
    );
}

extern "C" {
    fn is_listening() -> i32;
    fn get_flows_user(p: *mut u8) -> i32;
    fn get_flow_id(p: *mut u8) -> i32;
    fn get_event_body_length() -> i32;
    fn get_event_body(p: *mut u8) -> i32;
    fn set_error_log(p: *const u8, len: i32);
}

/// Create a cron job. Call the callback function with the content when the job is evoked.
///
/// `cron` is a [cron expression](https://crontab.guru/). There is
/// currently a limitation to use this function. The minute and hour
/// should be specified as exact number. That's to say, you can not
/// set `*`, value list or range of values to these two fields.
///
/// The bytes vec as the parameter of callback function is currently
/// the same as the body passed to the function.
pub fn schedule_cron_job<F>(cron: String, body: String, cb: F)
where
    F: Fn(Vec<u8>),
{
    unsafe {
        match is_listening() {
            // Calling register
            1 => {
                let mut flows_user = Vec::<u8>::with_capacity(100);
                let c = get_flows_user(flows_user.as_mut_ptr());
                flows_user.set_len(c as usize);
                let flows_user = String::from_utf8(flows_user).unwrap();

                let mut flow_id = Vec::<u8>::with_capacity(100);
                let c = get_flow_id(flow_id.as_mut_ptr());
                if c == 0 {
                    panic!("Failed to get flow id");
                }
                flow_id.set_len(c as usize);
                let flow_id = String::from_utf8(flow_id).unwrap();

                let mut writer = Vec::new();
                let res = request::post(
                    format!(
                        "{}/{}/{}/listen?cron={}",
                        SCHEDULE_API_PREFIX.as_str(),
                        flows_user,
                        flow_id,
                        urlencoding::encode(&cron)
                    ),
                    body.as_bytes(),
                    &mut writer,
                )
                .unwrap();

                match res.status_code().is_success() {
                    true => {}
                    false => {
                        set_error_log(writer.as_ptr(), writer.len() as i32);
                    }
                }
            }
            _ => {
                if let Some(m) = message_from_request() {
                    cb(m);
                }
            }
        }
    }
}

fn message_from_request() -> Option<Vec<u8>> {
    unsafe {
        let l = get_event_body_length();
        let mut event_body = Vec::<u8>::with_capacity(l as usize);
        let c = get_event_body(event_body.as_mut_ptr());
        assert!(c == l);
        event_body.set_len(c as usize);

        Some(event_body)
    }
}

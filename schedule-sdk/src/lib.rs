//! Make flow function acting as scheduled task in [Flows.network](https://test.flows.network)
//!
//! # Quick Start
//!
//! To get started, let's write a task that will send a message to Slack at a fixed time wach day.
//!
//! ```
//! use schedule_flows::cron_job_evoked;
//! use slack_flows::send_message_to_channel;
//!
//! #[no_mangle]
//! pub fn run() {
//!     if let Some(body) = cron_job_evoked(String::from("50 8 * * *"), String::from("cron_job_evoked")) {
//!         send_message_to_channel(
//!             "myworkspace",
//!             "mychannel",
//!             String::from_utf8_lossy(&body).into_owned(),
//!         );
//!     }
//! }
//! ```
//!
//! [cron_job_evoked] will create a cron job and return the content
//! when the job is triggered at 8:50 UTC each day.
use http_req::request;

const SCHEDULE_API_PREFIX: &str = "https://schedule-flows-extension.vercel.app/api";

extern "C" {
    fn is_listening() -> i32;
    fn get_flows_user(p: *mut u8) -> i32;
    fn get_flow_id(p: *mut u8) -> i32;
    fn get_event_body_length() -> i32;
    fn get_event_body(p: *mut u8) -> i32;
    fn set_error_log(p: *const u8, len: i32);
}

/// Create a cron job and return the content when the job is triggered.
///
/// `cron` is a [cron expression](https://crontab.guru/). There is
/// currently a limitation to use this function. The minute and hour
/// should be specified as exact number. That's to say, you can not
/// set `*`, value list or range of values to these two fields.
///
/// The return bytes vec is currently the same as the body passed to
/// the function.
pub fn cron_job_evoked(cron: String, body: String) -> Option<Vec<u8>> {
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
                        SCHEDULE_API_PREFIX,
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

                None
            }
            _ => message_from_request(),
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

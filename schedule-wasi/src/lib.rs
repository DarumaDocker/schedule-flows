use http_req::request;
use serde_json::Value;
use std::collections::HashMap;

const SCHEDULE_API_PREFIX: &str = "https://schedule-flows-extension.vercel.app/api";

extern "C" {
    fn get_event_query_length() -> i32;
    fn get_event_query(p: *mut u8) -> i32;
    fn set_flows(p: *const u8, len: i32);
}

#[no_mangle]
pub unsafe fn request() {
    let l = get_event_query_length();
    let mut event_query = Vec::<u8>::with_capacity(l as usize);
    let c = get_event_query(event_query.as_mut_ptr());
    assert!(c == l);
    event_query.set_len(c as usize);
    let event_query: HashMap<String, Value> = serde_json::from_slice(&event_query).unwrap();

    if let Some(l_key) = event_query.get("l_key") {
        let mut writer = Vec::new();
        let res = request::get(
            format!("{}/event/{}", SCHEDULE_API_PREFIX, l_key.as_str().unwrap()),
            &mut writer,
        )
        .unwrap();

        if res.status_code().is_success() {
            if let Ok(flows) = String::from_utf8(writer) {
                set_flows(flows.as_ptr(), flows.len() as i32);
            }
        }
    }
}

use http_req::request;
use serde_json::Value;
use std::collections::HashMap;

const SCHEDULE_API_PREFIX: &str = "http://127.0.0.1:3003/api";

extern "C" {
    fn get_flows_user(p: *mut u8) -> i32;
    fn get_flow_id(p: *mut u8) -> i32;
    fn get_event_query_length() -> i32;
    fn get_event_query(p: *mut u8) -> i32;
    fn get_event_body_length() -> i32;
    fn get_event_body(p: *mut u8) -> i32;
    fn set_flows(p: *const u8, len: i32);
    fn set_error_log(p: *const u8, len: i32);
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

        /*
        println!("Status: {} {}", res.status_code(), res.reason());
        println!("Headers {}", res.headers());
        println!("{}", String::from_utf8_lossy(&writer));
        */

        if res.status_code().is_success() {
            if let Ok(flows) = String::from_utf8(writer) {
                set_flows(flows.as_ptr(), flows.len() as i32);
            }
        }
    }
}

pub fn listen_to_request(cron: String, body: String) {
    unsafe {
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
                SCHEDULE_API_PREFIX, flows_user, flow_id, cron
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
}

pub fn message_from_request() -> Vec<u8> {
    unsafe {
        let l = get_event_body_length();
        let mut event_body = Vec::<u8>::with_capacity(l as usize);
        let c = get_event_body(event_body.as_mut_ptr());
        assert!(c == l);
        event_body.set_len(c as usize);

        event_body
    }
}

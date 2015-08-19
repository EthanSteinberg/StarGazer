extern crate hyper;
extern crate serde_json;
extern crate sqlite3;
extern crate starevent;

use hyper::Client;
use hyper::header::ContentType;
use serde_json::Value;
use sqlite3::DatabaseConnection;
use std::thread;

use starevent::*;

const KEY: &'static str = "603fae71ca2fb5a2c0b5c99262014089";
const EVENTS_URL: &'static str = "http://chat.stackoverflow.com/events";

fn get_data(starting_from: i64) -> Value {
    use std::io::Read;

    let body = format!("r10={}&fkey={}",starting_from, KEY);

    let client = Client::new();

    let mut res = client.post(EVENTS_URL)
        .header(ContentType::form_url_encoded())
        .body(&body)
        .send().unwrap();

    let mut buffer = String::new();
    res.read_to_string(&mut buffer).unwrap();

    let all_data: Value = serde_json::from_str(&buffer).unwrap();
    all_data.find("r10").unwrap().clone()
}


fn next_id(data: &Value) -> Option<i64> {
    data.find("t").and_then(|x| x.as_i64())
}

fn parse_event(event_data: Value) -> Option<StarEvent> {
    let event_type = event_data.find("event_type").and_then(|x| x.as_i64()).unwrap();
    match event_type {
        6 => Some(serde_json::from_value::<StarEvent>(event_data).unwrap()),
        _ => None,
    }
}

fn get_events(data: &Value) -> Option<Vec<StarEvent>> {
    data.find("e")
        .and_then(|x| x.as_array())
        .map(|x| x.iter()
            .flat_map(|val| parse_event(val.clone()))
            .collect())
}

fn process_event(conn: &mut DatabaseConnection, event: &StarEvent) { 
    if let Some(old_event) = get_star_for_message_and_user(conn, event.message_id, event.user_id) {
        if old_event.id == event.id {
            // Repeat event somehow?
//            println!("Ignoring star {:?}", event);
            return;
        } else {
            // This user has added a star, so this must be the remove star event
//            println!("Removing star {:?}", event);
            starevent::remove_star_from_db(conn, old_event.id);
        }
    } else {
//        println!("Adding star {:?}", event);
        // Mark the user as starring this item
        add_star_to_db(conn, event);
    }
}

fn main() {
    let mut current_id = 0;
    let mut conn = open_connection();

    loop {
        let data = get_data(current_id);
        // println!("{}", serde_json::to_string_pretty(&data).unwrap());
        
        let option_events = get_events(&data);
        if let Some(events) = option_events {
            for event in events {
                process_event(&mut conn, &event);
            }
        }

        current_id = next_id(&data).unwrap_or(current_id);
        thread::sleep_ms(4000);
    }
}
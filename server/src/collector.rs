extern crate hyper;
extern crate serde_json;
extern crate sqlite3;
extern crate starevent;
extern crate regex;

use hyper::Client;
use hyper::header::ContentType;
use serde_json::Value;
use sqlite3::DatabaseConnection;
use std::thread;
use std::collections::HashMap;

use starevent::*;

const EVENTS_URL: &'static str = "http://chat.stackoverflow.com/events";
const MAIN_URL: &'static str = "http://chat.stackoverflow.com/";

const ACTIVE_ROOMS: [i64; 2] = [10, 17];

fn get_key() -> String {
   use std::io::Read;

   let client = Client::new();

   let mut res = client.get(MAIN_URL)
       .send().unwrap();

   let mut buffer = String::new();
   res.read_to_string(&mut buffer).unwrap();

   let re = regex::Regex::new(r##"<input id="fkey" name="fkey" type="hidden" value="([a-f0-9]{32})" />"##).unwrap();
   let groups = re.captures(&buffer).unwrap();

   groups.at(1).unwrap().to_owned()
}

fn get_data(key: &str, room_counters: &HashMap<i64, i64>) -> Value {
    use std::io::Read;

    let body = format!("fkey={}&{}",key, get_all_rooms_string(room_counters));

    let client = Client::new();

    let mut res = client.post(EVENTS_URL)
        .header(ContentType::form_url_encoded())
        .body(&body)
        .send().unwrap();

    let mut buffer = String::new();
    res.read_to_string(&mut buffer).unwrap();

    serde_json::from_str(&buffer).unwrap()
}


fn next_id(data: &Value) -> Option<i64> {
    data.find("t").and_then(|x| x.as_i64())
}

fn get_room_string(room_id: i64) -> String {
    format!("r{}", room_id)
}

fn get_all_rooms_string(room_counters: &HashMap<i64, i64>) -> String {
    room_counters
        .iter()
        .map(|(room_id, room_counter)| format!("{}={}", get_room_string(*room_id), room_counter))
        .collect::<Vec<String>>()
        .join("&")
}

fn update_room_counters(data: &Value, room_counters: &mut HashMap<i64, i64>) {
    for (room_id, current_id) in room_counters.iter_mut() {
        let next_id = data.find(&get_room_string(*room_id)).and_then(|data| next_id(data));
        *current_id = next_id.unwrap_or(*current_id);
    }
}

fn parse_event(event_data: Value) -> Option<StarEvent> {
    let event_type = event_data.find("event_type").and_then(|x| x.as_i64()).unwrap();
    match event_type {
        6 => Some(serde_json::from_value::<StarEvent>(event_data).unwrap()),
        _ => None,
    }
}

fn get_events_for_room(data: &Value, room_id: i64) -> Vec<StarEvent> {
    data.find(&get_room_string(room_id))
        .and_then(|data| data.find("e"))
        .and_then(|x| x.as_array())
        .map(|x| x.iter()
            .flat_map(|val| parse_event(val.clone()))
            .collect())
        .unwrap_or(Vec::new())
}

fn get_events(data: &Value) -> Vec<StarEvent> {
    ACTIVE_ROOMS.iter().flat_map(|room_id| get_events_for_room(data, *room_id)).collect()
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
    let mut room_counters: HashMap<i64, i64> = ACTIVE_ROOMS.iter().map(|room| (*room, 0)).collect();
    let mut conn = open_connection();

    let key = get_key();

    loop {
        let data = get_data(&key, &room_counters);
        // println!("{}", serde_json::to_string_pretty(&data).unwrap());
        
        for event in get_events(&data) {
            process_event(&mut conn, &event);
        }

        update_room_counters(&data, &mut room_counters);
        thread::sleep_ms(4000);
    }
}
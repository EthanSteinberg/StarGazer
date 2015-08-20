#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;
extern crate sqlite3;

use sqlite3::access;
use sqlite3::{DatabaseConnection, ResultRowAccess, ResultRow};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: i64,
    pub room_id: i64,
    pub time_stamp: i64,
    pub user_id: i64,
    pub user_name: String,

    pub event_type: Option<i64>,
    pub content: Option<String>,
    pub message_id: Option<i64>,

    pub room_name: Option<String>,
    pub parent_id: Option<i64>,
    pub show_parent: Option<bool>,
    pub message_stars: Option<i64>,
    pub message_edits: Option<i64>,
    pub message_owner_stars: Option<i64>,
    pub target_user_id: Option<i64>,
}

fn convert_result_to_star(mut result: ResultRow) -> Event {
    Event {
        id: result.get(0),
        message_id: result.get(1),
        room_id: result.get(2),
        time_stamp: result.get(3),
        user_id: result.get(4),
        user_name: result.get(5),

        event_type: None,
        room_name: None,
        show_parent: None,
        message_stars: None,
        content: None,
        parent_id: None,
        message_edits: None,
        message_owner_stars: None,
        target_user_id: None,
    }
}

pub fn get_star_for_message_and_user(conn: &mut DatabaseConnection, message_id: i64, user_id: i64) -> Option<Event> {
    let sql = "SELECT id, message_id, room_id, time_stamp, user_id, user_name FROM stars where message_id = $1 AND user_id = $2";
    let mut stmt = conn.prepare(sql).unwrap();
    stmt.bind_int64(1, message_id).unwrap();
    stmt.bind_int64(2, user_id).unwrap();
    let mut result_set = stmt.execute();
    result_set.step().unwrap().map(|row| convert_result_to_star(row))
}

pub fn get_stars_for_message(conn: &mut DatabaseConnection, message_id: i64) -> Vec<Event> {
    let sql = "SELECT id, message_id, room_id, time_stamp, user_id, user_name FROM stars where message_id = $1";
    let mut stmt = conn.prepare(sql).unwrap();
    stmt.bind_int64(1, message_id).unwrap();
    let mut result_set = stmt.execute();
    let mut result_events = Vec::new();
    loop {
        let next_row = result_set.step().unwrap();
        match next_row {
            Some(row) => {
                let star = convert_result_to_star(row);
                result_events.push(star);
            }

            None => return result_events,
        }
    }
}

pub fn remove_star_from_db(conn: &mut DatabaseConnection, id: i64) {
    let sql = "DELETE FROM stars where id = $1";
    let mut stmt = conn.prepare(sql).unwrap();
    stmt.bind_int64(1, id).unwrap();
    let mut result_set = stmt.execute();
    assert!(result_set.step().unwrap().is_none());
}

pub fn add_star_to_db(conn: &mut DatabaseConnection, star: &Event) {
    let sql = "INSERT INTO stars (id, message_id, room_id, time_stamp, user_id, user_name) VALUES ($1, $2, $3, $4, $5, $6)";
    let mut stmt = conn.prepare(sql).unwrap();
    stmt.bind_int64(1, star.id).unwrap();
    stmt.bind_int64(2, star.message_id.unwrap()).unwrap();
    stmt.bind_int64(3, star.room_id).unwrap();
    stmt.bind_int64(4, star.time_stamp).unwrap();
    stmt.bind_int64(5, star.user_id).unwrap();
    stmt.bind_text(6, &star.user_name).unwrap();
    let mut result_set = stmt.execute();
    assert!(result_set.step().unwrap().is_none());
}

pub fn update_message(conn: &mut DatabaseConnection, message_event:& Event) {
    let sql = "INSERT OR REPLACE INTO messages (id, content) VALUES ($1, $2)";
    let mut stmt = conn.prepare(sql).unwrap();
    stmt.bind_int64(1, message_event.message_id.unwrap()).unwrap();
    stmt.bind_text(2, message_event.content.as_ref().unwrap()).unwrap();
    let mut result_set = stmt.execute();
    assert!(result_set.step().unwrap().is_none());
}

pub fn get_message_content(conn: &mut DatabaseConnection, message_id: i64) -> Option<String> {
    let sql = "SELECT content FROM messages where id = $1";
    let mut stmt = conn.prepare(sql).unwrap();
    stmt.bind_int64(1, message_id).unwrap();
    let mut result_set = stmt.execute();
    result_set.step().unwrap().map(|mut row| row.get(0))
}

fn get_database_path() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.push("../../data/stars.db");
    path.to_str().unwrap().to_owned()
}

pub fn open_connection() -> DatabaseConnection {
    access::open(&get_database_path(), Default::default()).unwrap()
}
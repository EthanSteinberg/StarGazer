extern crate hyper;
extern crate serde_json;
extern crate sqlite3;
extern crate starevent;
extern crate url;

use hyper::net::Fresh;
use hyper::server::Request;
use hyper::server::Response;
use hyper::Server;
use hyper::uri::RequestUri;
use hyper::header::AccessControlAllowOrigin;
use sqlite3::DatabaseConnection;
use std::collections::HashMap;

use starevent::*;

fn get_stars_from_query(conn: &mut DatabaseConnection, query: Option<String>) -> Option<Vec<StarEvent>> {
    query
        .map(|query| url::form_urlencoded::parse(query.as_bytes()).into_iter().collect::<HashMap<String, String>>())
        .and_then(|params| params.get("message_id").cloned())
        .and_then(|message_id_str| message_id_str.parse::<i64>().ok())
        .map(|message_id| get_stars_for_message(conn, message_id))
}

fn get_stars(conn: &mut DatabaseConnection, req: Request) -> Option<Vec<StarEvent>> {
    let uri = match req.uri {
        RequestUri::AbsolutePath(a) => Some(a),
        _ => None
    };

    uri
        .and_then(|uri| url::parse_path(&uri).ok())
        .and_then(|(path, query, _)| if path == vec!["stars"] { Some(query) } else { None})
        .and_then(|query| get_stars_from_query(conn, query))
}

fn hello(req: Request, mut res: Response<Fresh>) {
    let mut conn = open_connection();

    if let Some(stars) = get_stars(&mut conn, req) {
        let stars_json = serde_json::to_string_pretty(&stars).unwrap();
        res.headers_mut().set(AccessControlAllowOrigin::Any);
        res.send(stars_json.as_bytes()).unwrap();
    } else {
        res.send(b"Invalid params").unwrap();
    }
   
}

fn main() {
    Server::http("localhost:3000").unwrap().handle(hello).unwrap();
}
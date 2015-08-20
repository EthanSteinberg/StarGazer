#![feature(slice_patterns)]

extern crate hyper;
extern crate serde_json;
extern crate sqlite3;
extern crate util;
extern crate url;

use hyper::net::Fresh;
use hyper::server::Request;
use hyper::server::Response;
use hyper::Server;
use hyper::uri::RequestUri;
use hyper::header::AccessControlAllowOrigin;
use sqlite3::DatabaseConnection;
use std::collections::HashMap;

use util::*;

fn parse_query(query: String) -> HashMap<String, String> {
    url::form_urlencoded::parse(query.as_bytes()).into_iter().collect()
}

fn get_path_and_query(req: Request) -> Option<(Vec<String>, HashMap<String, String>)> {
    let uri = match req.uri {
        RequestUri::AbsolutePath(a) => Some(a),
        _ => None
    };

    uri
        .and_then(|uri| url::parse_path(&uri).ok())
        .map(|(path, query, _)| (path, query.map(parse_query)))
        .and_then(|(path, params)| if let Some(params) = params { Some((path, params)) } else { None })
}

fn get_message_id(params: &HashMap<String, String>) -> Option<i64> {
    params.get("message_id").cloned()
        .and_then(|message_id_str| message_id_str.parse::<i64>().ok())
}

fn get_stars(conn: &mut DatabaseConnection, params: &HashMap<String, String>) -> Option<String> {
    get_message_id(&params)
        .map(|message_id| get_stars_for_message(conn, message_id))
        .and_then(|stars| serde_json::to_string_pretty(&stars).ok())
}

fn get_message(conn: &mut DatabaseConnection, params: &HashMap<String, String>) -> Option<String> {
    get_message_id(&params)
        .map(|message_id| get_message_content(conn, message_id).unwrap_or("No message".to_owned()))
}

fn main_handler(path: &Vec<String>, params: &HashMap<String, String>) -> Option<String> {
    let mut conn = open_connection();

    if *path == vec!["stars"] {
        get_stars(&mut conn, &params)
    } else if *path == vec!["message"] {
        get_message(&mut conn, &params)
    } else {
        None
    }
}

fn hello(req: Request, mut res: Response<Fresh>) {
    let result = 
        get_path_and_query(req)
            .and_then(|(path, params)| main_handler(&path, &params))
            .unwrap_or("Invalid path or parameters".to_owned());

    res.headers_mut().set(AccessControlAllowOrigin::Any);
    res.send(result.as_bytes()).unwrap();
}

fn main() {
    Server::http("localhost:3000").unwrap().handle(hello).unwrap();
}
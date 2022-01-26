use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ptr::write;
use std::path::Path;

use handlebars::Handlebars;
use lazy_static::lazy_static;
use querystring::QueryParams;
use regex::Regex;
use serde_json::{json, Value};
use substring::Substring;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Connection established! {}", stream.peer_addr().unwrap());
                handle_connection(stream);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();
    let request_as_string = String::from_utf8_lossy(&buffer[..]);

    lazy_static! {
        static ref PATH_REGEX: Regex = Regex::new("GET (/[0-9A-z/]*)(\\?[0-9A-z=]*)? HTTP/1\\.1").unwrap();
    }
    let mut path = "unset";
    let mut query = "unset";

    let capture = PATH_REGEX.captures(&request_as_string);

    match capture {
        Some(capture) => {
            path = capture.get(1).unwrap().as_str();
            let query_capture = capture.get(2);
            if query_capture != None {
                query = query_capture.unwrap().as_str();
            }
        }
        None => {
            // Sometimes the request body is empty and the capture returns None
            // This will happen about 1 minute after the last page load, not sure what is causing it.
            println!("Error: request body invalid");
            return;
        }
    }

    let mut query_params_json: Value = json!({});

    // If a query string was provided, turn it into a json object for the template
    if query.len() > 1 {
        let query_params = querystring::querify(query.substring(1, query.len()));
        query_params_json = params_to_json(query_params);
    }

    let file_path= normalize_filepath(path);

    // Load the template file if it exists, otherwise return a 404
    if Path::new(&file_path).exists() {
        let reg = Handlebars::new();
        let template = fs::read_to_string(file_path).unwrap();
        let content = reg.render_template(&template, &query_params_json).unwrap();
        write_response(stream, 200, &content);
    } else {
        let contents = fs::read_to_string("src/pages/404.html").unwrap();
        write_response(stream,404, &contents);
    }
}

// Converts /route/home to /route/home.hbs or /route/home/index.hbs
fn normalize_filepath(request_path: &str) -> String {
    let mut additional_path = "";
    if request_path.ends_with("/") {
        additional_path = "index.hbs";
    } else {
        additional_path = ".hbs";
    }
    let mut path = String::from("src/pages");
    path.push_str(request_path);
    path.push_str(additional_path);
    return path;
}

fn write_response(mut stream: TcpStream, status_code: u16, contents: &str) {
    lazy_static! {
        static ref STATUS_CODE_TO_STATUS_STRING: HashMap<u16, &'static str> = HashMap::from([
            (200, "200 OK"),
            (404, "404 NOT FOUND"),
        ]);
    }

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        STATUS_CODE_TO_STATUS_STRING.get(&status_code).unwrap(),
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn params_to_json(params: QueryParams) -> Value {
    let mut json_params = json!({});

    for param in params {
        json_params[param.0] = serde_json::to_value(param.1).unwrap();
    }

    return json_params;
}

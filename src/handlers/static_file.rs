use std::collections::HashMap;
use std::fs;

use crate::http::request::Version;
use crate::http::response::StatusCode;

#[derive(Debug)]
pub enum StaticFileError {
    BodyResponseError,
    HeaderResponseError,
    StatusLineResponseError,
}

pub fn body(path: &str) -> Result<Vec<u8>, StaticFileError> {

    let system_path = format!("public{}", path);

    match fs::read(system_path) {
        Ok(raw_file_bytes) => {
            Ok(raw_file_bytes)
        },
        Err(_e) => {
            Err(StaticFileError::BodyResponseError)
        }
    }
}

pub fn headers(path: &str, body: &Result<Vec<u8>, StaticFileError>) -> Result<HashMap<String, String>, StaticFileError> {

    let mut temp_header: HashMap<String, String> = HashMap::new();
    
    // content type
    let file_type: Vec<&str> = path.split(".").collect();

    let mime = match file_type[1] {
    "html" => "text/html",
    "css" => "text/css",
    "js" => "application/javascript",
    "json" => "application/json",
    "png" => "image/png",
    "jpg" | "jpeg" => "image/jpeg",
    "svg" => "image/svg+xml",
    "ico" => "image/x-icon",
    "txt" => "text/plain",
    _ => return Err(StaticFileError::HeaderResponseError),
    };

    temp_header.insert(
        "Content-Type".to_string(),
        mime.to_string(),
    );

    // content length
    match body {
        Ok(body) => {
            let length = body.len();

            temp_header.insert(
                "Content-Length".to_string(),
                length.to_string()
            );
        },
        Err(_e) => {
            temp_header.insert(
                "Content-Length".to_string(),
                "0".to_string()
            );
        }
    }
    

    // connection
    temp_header.insert(
        "Connection".to_string(),
        "close".to_string()
    );

    // server
    temp_header.insert(
        "Server".to_string(),
        "Rustinx".to_string()
    );

    Ok(temp_header)
}

pub fn status_line(version: &Version, status: StatusCode) -> String {
    format!(
        "{} {} {}\r\n",
        version.as_str(),
        status.code(),
        status.reason(),
    )
}
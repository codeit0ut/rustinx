use std::collections::HashMap;

pub enum StatusCode {
    Ok,                  // 200
    Created,             // 201
    MovedPermanently,    // 301
    BadRequest,          // 400
    Forbidden,           // 403
    NotFound,            // 404
    MethodNotAllowed,    // 405
    InternalServerError, // 500
    BadGateway,          // 502
    ServiceUnavailable,  // 503
}

impl StatusCode {
    pub fn code(&self) -> u16 {
        match self {
            StatusCode::Ok => 200,
            StatusCode::Created => 201,
            StatusCode::MovedPermanently => 301,
            StatusCode::BadRequest => 400,
            StatusCode::Forbidden => 403,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::InternalServerError => 500,
            StatusCode::BadGateway => 502,
            StatusCode::ServiceUnavailable => 503,
        }
    }

    pub fn reason(&self) -> &str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::Created => "Created",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::BadGateway => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
        }
    }
}

pub struct Response {
    pub status_line: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}
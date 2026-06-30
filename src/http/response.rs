use std::{collections::HashMap, str::from_utf8};

use crate::http::request::{ParseError, Version};

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
    Other(u16),
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
            StatusCode::Other(code) => *code,
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
            StatusCode::Other(_) => "Unknown",
        }
    }
}

pub struct Response {
    pub version: Version,
    pub status_code: StatusCode,
    pub reason: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn parser(raw_bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor: usize = 0;

        let version = parse_version(raw_bytes, &mut cursor)?;
        let status_code = parse_status_code(raw_bytes, &mut cursor)?;
        let reason = parse_reason(raw_bytes, &mut cursor)?;

        let headers = parse_headers(raw_bytes, &mut cursor)?;
        let content_length: usize = headers
            .get("Content-Length")
            .unwrap_or(&"0".to_string())
            .parse::<usize>()
            .map_err(|_| ParseError::InvalidContentLength)?;

        let mut body: Vec<u8> = Vec::new();
        if content_length > 0 {
            body = parse_body(raw_bytes, &mut cursor, content_length)?;
        }

        Ok(Response {
            version,
            status_code,
            reason,
            headers,
            body,
        })
    }

    pub fn header_parser(raw_bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor: usize = 0;

        let version = parse_version(raw_bytes, &mut cursor)?;
        let status_code = parse_status_code(raw_bytes, &mut cursor)?;
        let reason = parse_reason(raw_bytes, &mut cursor)?;

        let headers = parse_headers(raw_bytes, &mut cursor)?;

        let body: Vec<u8> = Vec::new();

        Ok(Response {
            version,
            status_code,
            reason,
            headers,
            body,
        })
    }
}

fn parse_version(raw_bytes: &[u8], cursor: &mut usize) -> Result<Version, ParseError> {
    let start = *cursor;
    while raw_bytes.len() > *cursor && raw_bytes[*cursor] != b' ' {
        *cursor += 1;
    }

    let temp_version =
        from_utf8(&raw_bytes[start..*cursor]).map_err(|_| ParseError::InvalidVersion)?;

    let version: Version = match temp_version {
        "HTTP/1.1" => Version::Http11,
        "HTTP/1.0" => Version::Http10,
        _ => return Err(ParseError::InvalidVersion),
    };

    *cursor += 1;

    Ok(version)
}

fn parse_status_code(raw_bytes: &[u8], cursor: &mut usize) -> Result<StatusCode, ParseError> {
    let start = *cursor;
    while raw_bytes.len() > *cursor && raw_bytes[*cursor] != b' ' {
        *cursor += 1;
    }

    let temp_status_code =
        from_utf8(&raw_bytes[start..*cursor]).map_err(|_| ParseError::InvalidMethod)?;

    let status_code: StatusCode = match temp_status_code {
        "200" => StatusCode::Ok,
        "201" => StatusCode::Created,
        "301" => StatusCode::MovedPermanently,
        "400" => StatusCode::BadRequest,
        "403" => StatusCode::Forbidden,
        "404" => StatusCode::NotFound,
        "405" => StatusCode::MethodNotAllowed,
        "500" => StatusCode::InternalServerError,
        "502" => StatusCode::BadGateway,
        "503" => StatusCode::ServiceUnavailable,
       _ => {
            let code = temp_status_code
                .parse::<u16>()
                .map_err(|_| ParseError::InvalidStatusCode)?;
            StatusCode::Other(code)
        }
    };

    *cursor += 1;

    Ok(status_code)
}

fn parse_reason(raw_bytes: &[u8], cursor: &mut usize) -> Result<String, ParseError> {
    let start = *cursor;
    while raw_bytes.len() > *cursor && raw_bytes[*cursor] != b'\r' {
        *cursor += 1;
    }
    let reason = from_utf8(&raw_bytes[start..*cursor])
        .map_err(|_| ParseError::InvalidReason)?
        .to_string();

    if &raw_bytes[*cursor..*cursor+4] == b"\r\n\r\n" {
        *cursor += 0;
    } else {
        *cursor += 2;
    }
    Ok(reason)
}

fn parse_headers(
    raw_bytes: &[u8],
    cursor: &mut usize,
) -> Result<HashMap<String, String>, ParseError> {
    let start = *cursor;
    *cursor = raw_bytes.windows(4).position(|w| w == b"\r\n\r\n")
        .ok_or(ParseError::UnexpectedEndOfRequest)?;
    let temp_headers =
        from_utf8(&raw_bytes[start..*cursor]).map_err(|_| ParseError::InvalidUtf8)?;
    let mut headers = HashMap::new();

    let lines: Vec<&str> = temp_headers.split("\r\n").collect();

    for line in lines {
        let (header_name, header_value) = line.split_once(':').ok_or(ParseError::InvalidHeader)?;

        let header_value = header_value.trim();

        headers.insert(header_name.to_string(), header_value.to_string());
    }

    *cursor += 4;

    Ok(headers)
}

fn parse_body(
    raw_bytes: &[u8],
    cursor: &mut usize,
    content_length: usize,
) -> Result<Vec<u8>, ParseError> {
    if content_length > 0 && raw_bytes.len() >= *cursor + content_length {
        let body = raw_bytes[*cursor..*cursor + content_length].to_vec();

        Ok(body)
    } else {
        Ok(Vec::new())
    }
}

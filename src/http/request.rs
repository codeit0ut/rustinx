use std::collections::HashMap;
use std::str::from_utf8;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEndOfRequest,
    InvalidUtf8,
    InvalidMethod,
    InvalidPath,
    InvalidVersion,
    InvalidHeader,
    MissingContentLength,
    InvalidContentLength,
    InvalidReason,
    InvalidStatusCode,
}

#[derive(Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

#[derive(Debug)]
pub enum Version {
    Http10,
    Http11,
}

impl Version {
    pub fn as_str(&self) -> &str {
        match self {
            Version::Http10 => "HTTP/1.0",
            Version::Http11 => "HTTP/1.1",
        }
    }
}

pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: Vec<(String, Option<String>)>,
    pub version: Version,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl Request {
    pub fn parser(raw_bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor: usize = 0;
        let mut has_query: bool = false;

        let method:Method = parse_method(raw_bytes, &mut cursor)?;
        let path: String = parse_path(raw_bytes, &mut cursor, &mut has_query)?;

        let mut query: Vec<(String, Option<String>)> = vec![];
        if has_query {
            query = parse_query(raw_bytes, &mut cursor)?;
        }

        let version: Version = parse_version(raw_bytes, &mut cursor)?;
        let headers: HashMap<String, String> = parse_headers(raw_bytes, &mut cursor)?;
        let content_length: usize =  headers.get("Content-Length")
            .unwrap_or(&"0".to_string())
            .parse::<usize>()
            .map_err(|_| ParseError::InvalidContentLength)?;

        let mut body = String::new();
        if content_length > 0 {
            body = parse_body(raw_bytes, &mut cursor, content_length)?;
        }

        Ok(Request { method, path, query, version, headers, body })
    }

    pub fn header_parser(raw_bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor: usize = 0;
        let mut has_query: bool = false;

        let method:Method = parse_method(raw_bytes, &mut cursor)?;
        let path: String = parse_path(raw_bytes, &mut cursor, &mut has_query)?;

        let mut query: Vec<(String, Option<String>)> = vec![];
        if has_query {
            query = parse_query(raw_bytes, &mut cursor)?;
        }

        let version: Version = parse_version(raw_bytes, &mut cursor)?;
        let headers: HashMap<String, String> = parse_headers(raw_bytes, &mut cursor)?;
        let body = String::new();

        Ok(Request { method, path, query, version, headers, body })
    }
}

fn parse_method(raw_bytes: &[u8], cursor: &mut usize) -> Result<Method, ParseError> {
    let start = *cursor;
    while raw_bytes.len() > *cursor && raw_bytes[*cursor] != b' ' {
        *cursor += 1;
    }

    let temp_method = from_utf8(&raw_bytes[start..*cursor])
        .map_err(|_| ParseError::InvalidUtf8)?;

    let method: Method = match temp_method {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "OPTIONS" => Method::OPTIONS,
        "HEAD" => Method::HEAD,
        "PATCH" => Method::PATCH,
        _ => return Err(ParseError::InvalidMethod)
    };

    *cursor += 1;

    Ok(method)
}

fn parse_path(raw_bytes: &[u8], cursor: &mut usize, has_query: &mut bool) -> Result<String, ParseError> {
    let start = *cursor;
    while raw_bytes.len() > *cursor && raw_bytes[*cursor] != b'?' && raw_bytes[*cursor] != b' ' {
        *cursor += 1;
    }

    if *cursor >= raw_bytes.len() {
        return Err(ParseError::InvalidPath);
    }

    if raw_bytes[*cursor] == b'?' {
        let path = from_utf8(&raw_bytes[start..*cursor])
            .map_err(|_| ParseError::InvalidUtf8)?
            .to_string();
        *cursor += 1;
        *has_query = true;

        Ok(path)
    } else if raw_bytes[*cursor] == b' ' {
        let path = from_utf8(&raw_bytes[start..*cursor])
            .map_err(|_| ParseError::InvalidUtf8)?
            .to_string();
        *cursor += 1;

        Ok(path)
    } else {
        Err(ParseError::InvalidPath)
    }
}

fn parse_query(raw_bytes: &[u8], cursor: &mut usize) -> Result<Vec<(String, Option<String>)>, ParseError> {
    let start = *cursor;
    let mut params: Vec<(String, Option<String>)> = vec![];
    while raw_bytes.len() > *cursor && raw_bytes[*cursor] != b' ' {
        *cursor += 1;
    }

    let query = from_utf8(&raw_bytes[start..*cursor])
        .map_err(|_| ParseError::InvalidUtf8)?
        .to_string();

    for pair in query.split("&") {
        if let Some((k, v)) = pair.split_once("=") {
            params.push((k.to_string(), Some(v.to_string())));
        } else {
            params.push((pair.to_string(), None));
        }
    }
    *cursor += 1;

    Ok(params)
}

fn parse_version(raw_bytes: &[u8], cursor: &mut usize) -> Result<Version, ParseError> {
    let start = *cursor;
    while raw_bytes.len() > *cursor && raw_bytes[*cursor] != b'\r' {
        *cursor += 1;
    }

    let temp_version = from_utf8(&raw_bytes[start..*cursor])
        .map_err(|_| ParseError::InvalidUtf8)?;

    let version: Version = match temp_version {
        "HTTP/1.0" => Version::Http10,
        "HTTP/1.1" => Version::Http11,
        _ => return Err(ParseError::InvalidVersion),
    };

    *cursor += 2;

    Ok(version)
}

fn parse_headers(raw_bytes: &[u8], cursor: &mut usize) -> Result<HashMap<String, String>, ParseError> {
    let start = *cursor;
    *cursor = raw_bytes.windows(4).position(|w| w == b"\r\n\r\n")
        .ok_or(ParseError::UnexpectedEndOfRequest)?;
    let temp_headers = from_utf8(&raw_bytes[start..*cursor])
        .map_err(|_| ParseError::InvalidUtf8)?;
    let mut headers = HashMap::new(); 

    let lines: Vec<&str> = temp_headers.split("\r\n").collect();

    for line in lines {
        let (header_name, header_value) = line.split_once(':')
            .ok_or(ParseError::InvalidHeader)?;

        let header_value = header_value.trim();

        headers.insert(
            header_name.to_string(),
            header_value.to_string(),
        );
    }

    *cursor += 4;

    Ok(headers)
}

fn parse_body(raw_bytes: &[u8], cursor: &mut usize, content_length: usize) -> Result<String, ParseError> {
    if content_length > 0 && raw_bytes.len() >= *cursor + content_length {
        let body =  from_utf8(&raw_bytes[*cursor..*cursor+content_length])
            .map_err(|_| ParseError::InvalidUtf8)?
            .to_string();

        Ok(body)
    } else {
        Err(ParseError::UnexpectedEndOfRequest)
    }
}
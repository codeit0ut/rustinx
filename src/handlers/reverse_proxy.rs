use std::net::TcpStream;
use std::io::{Write, Read};

use crate::http::request::{Request, Method, Version};

#[derive(Debug)]
pub enum ProxyError {
    ConnectionFailed,
    WriteFailed,
    ReadFailed,
    InvalidResponse,
}

#[derive(Debug)]
pub enum ConversionError {
    ToRawBytesFailed,
    ToStringFailed,
}

pub fn request_to_bytes(request: &Request) -> Vec<u8> {
    let mut raw_request: Vec<u8> = Vec::new();

    let method = match request.method {
        Method::GET => "GET",
        Method::POST => "POST",
        Method::PUT => "PUT",
        Method::DELETE => "DELETE",
        Method::OPTIONS => "OPTIONS",
        Method::HEAD => "HEAD",
        Method::PATCH => "PATCH",
    };

    raw_request.extend_from_slice(method.as_bytes());

    raw_request.push(b' ');

    raw_request.extend_from_slice(request.path.as_bytes());

    if request.query.len() > 0 {
        raw_request.push(b'?');
        for (i, (key, value)) in request.query.iter().enumerate() {
            raw_request.extend_from_slice(key.as_bytes());

            if let Some(v) = value {
                raw_request.push(b'=');
                raw_request.extend_from_slice(v.as_bytes());
            }

            if i != request.query.len() - 1 {
                raw_request.push(b'&');
            }
        }
        raw_request.push(b' ');
    } else {
        raw_request.push(b' ');
    }

    let version = match request.version {
        Version::Http10 => "HTTP/1.0",
        Version::Http11 => "HTTP/1.1",
    };

    raw_request.extend_from_slice(version.as_bytes());

    raw_request.extend_from_slice("\r\n".as_bytes());

    for (key, value) in &request.headers {
        raw_request.extend_from_slice(key.as_bytes());
        raw_request.extend_from_slice(": ".as_bytes());
        raw_request.extend_from_slice(value.as_bytes());
        raw_request.extend_from_slice("\r\n".as_bytes());
    }

    raw_request.extend_from_slice("\r\n\r\n".as_bytes());

    raw_request.extend_from_slice(request.body.as_bytes());

    raw_request

}

pub fn get_connection() -> Result<TcpStream, ProxyError> {
    let upstream = TcpStream::connect("localhost:5173")
        .map_err(|_| ProxyError::ConnectionFailed)?;

    Ok(upstream)
}

pub fn write_request(upstream: &mut TcpStream, request: &Request) -> Result<(), ProxyError> {
    let request = request_to_bytes(request);

    upstream.write_all(&request)
        .map_err(|_| ProxyError::WriteFailed)?;

    Ok(())
}

pub fn read_response(upstream: &mut TcpStream) -> Result<Vec<u8>, ProxyError> {
    let mut response = Vec::new();

    upstream.read_to_end(&mut response)
        .map_err(|_| ProxyError::ReadFailed)?;

    Ok(response)
}
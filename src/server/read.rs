use std::net::TcpStream;
use std::io::Read;

use crate::http::request::Request;

#[derive(Debug)]
pub enum ReadError {
    ConnectionClosed,
    InvalidRequest,
    IoError,
}

pub fn read_request(stream: &mut TcpStream) -> Result<Vec<u8>, ReadError> {
    let mut buffer = [0u8; 1024];
    let mut request = Vec::new();

    loop {
        // let bytes_read = stream.read(&mut buffer)
        //     .map_err(|_| ReadError::StreamReadError)?;

        let bytes_read = stream.read(&mut buffer)
            .map_err(|_e| {
                ReadError::InvalidRequest
            })?;

        if bytes_read == 0 && request.is_empty() {
            return Err(ReadError::ConnectionClosed);
        }

        request.extend_from_slice(&buffer[..bytes_read]);

        if request.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    
    let headers = Request::header_parser(&request)
        .map_err(|_| ReadError::InvalidRequest)?;

    let header_end = request
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .unwrap()
        + 4;

    let content_length = match headers.headers.get("Content-Length") {
        Some(v) => v.parse().map_err(|_| ReadError::InvalidRequest)?,
        None => 0,
    };

    while request.len() - header_end < content_length {
        let bytes_read = stream.read(&mut buffer)
            .map_err(|_| ReadError::InvalidRequest)?;

        if bytes_read == 0 {
            return Err(ReadError::InvalidRequest);
        }

        request.extend_from_slice(&buffer[..bytes_read]);
    }

    Ok(request)
}
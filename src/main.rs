use std::io::{Read, Write};
use std::net::TcpListener;

use rustinx::http::request::Request;
use rustinx::http::response::{Response, StatusCode};
use rustinx::router::route::{RouteTarget, route_resolver};
use rustinx::handlers::static_file;
use rustinx::handlers::reverse_proxy::{get_connection, write_request, read_response};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .expect("Failed to bind address");

    println!("Server running on 127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection established!");

                let mut buffer = [0; 1024];
               
                match stream.read(&mut buffer) {
                    Ok(bytes_read) => {
                        let req = Request::parser(&buffer[..bytes_read]).unwrap();

                        let req_type = route_resolver(&req.path).unwrap();

                        match req_type {
                            RouteTarget::Static => {
                                let body = static_file::body(&req.path);
                                let headers = static_file::headers(&req.path, &body).expect("not good");
                                let status = match &body {
                                    Ok(_) => StatusCode::Ok,
                                    Err(_) => StatusCode::NotFound,
                                };

                                let internal_response = Response {
                                        version: req.version,
                                        status_code: StatusCode::Other(status.code()),
                                        reason: status.reason().to_string(),
                                        headers: headers,
                                        body: body.unwrap_or(Vec::new()),
                                    };

                                let status_line = format!(
                                    "{} {} {}",
                                    internal_response.version.as_str(),
                                    internal_response.status_code.code(),
                                    internal_response.reason
                                );
                                
                                let mut response = Vec::new();

                                response.extend_from_slice(status_line.as_bytes());

                                for (key, value) in &internal_response.headers {
                                    response.extend_from_slice(
                                        format!("{}: {}\r\n", key, value).as_bytes()
                                    );
                                }

                                response.extend_from_slice(b"\r\n");

                                // body
                                response.extend_from_slice(&internal_response.body);

                                stream.write_all(&response).expect("Failed to write to stream")
                            },
                            RouteTarget::Proxy => {
                                let mut upstream = get_connection().unwrap();

                                write_request(&mut upstream, &req).unwrap();

                                let response = read_response(&mut upstream).unwrap();
                                println!("{}", response.len());

                                stream.write_all(&response).expect("Failed to write to stream")
                            },
                        }

                    }
                    Err(e) => {
                        println!("Failed to read from client: {}", e);
                    }
                }
                
            }
            Err(e) => { 
                println!("Connection failed: {}", e);
            }
        }
    }
}

use std::io::{Read, Write};
use std::net::TcpListener;

use rustinx::http::request::Request;
use rustinx::http::response::{Response, StatusCode};
use rustinx::router::route::{RouteTarget, route_resolver};
use rustinx::handlers::static_file;

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
                        println!("Received {} bytes", bytes_read);
                        let req = Request::parser(&buffer[..bytes_read]).unwrap();

                        let req_type = route_resolver(&req.path).unwrap();

                        match req_type {
                            RouteTarget::Static => {
                                let body = static_file::body(&req.path);
                                let headers = static_file::headers(&req.path, &body).expect("not good");
                                let status_line = match &body {
                                    Ok(_) => static_file::status_line(&req.version, StatusCode::Ok),
                                    Err(_) => static_file::status_line(&req.version, StatusCode::NotFound),
                                };

                                let internal_response = Response {
                                        status_line: status_line,
                                        headers: headers,
                                        body: body.unwrap_or(Vec::new()),
                                    };
                                
                                let mut response = Vec::new();

                                response.extend_from_slice(internal_response.status_line.as_bytes());

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

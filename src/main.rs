use std::io::{Read, Write};
use std::net::TcpListener;
use std::fs;

use rustinx::http::request::Request;

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
                        let test = Request::parser(&buffer[..bytes_read]);

                        println!("Ram Ram {}", test.method);
                        println!("Ram Ram {}", test.path);
                        println!("Ram Ram {}", test.version);

                        for (name, value) in test.headers {
                            println!("Ram Ram - {}: {}", name, value);
                        }

                        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                        
                        println!("Message: {}", message);

                        let request_line = message.lines().next().unwrap();
                        let line_parts: Vec<&str> = request_line.split_whitespace().collect();
                        let url_path = line_parts[1];
                        let file_path = if url_path == "/" {
                            "public/index.html".to_string()
                        } else {
                            format!("public{}", url_path)
                        };
                        let file_extension: Vec<&str> = file_path.split(".").collect();
                        let mime = if file_extension[1] == "css" {
                            "Content-Type: text/css".to_string()
                        } else if file_extension[1] == "js" {
                            "Content-Type: application/js".to_string()
                        } else if file_extension[1] == "json" {
                            "Content-Type: application/json".to_string()
                        } else if file_extension[1] == "png" {
                            "Content-Type: image/png".to_string()
                        } else if file_extension[1] == "jpeg" {
                            "Content-Type: image/jpeg".to_string()
                        } else {
                            "Content-Type: text/html".to_string()
                        };

                        if file_extension[1] == "png" || file_extension[1] == "jpeg" {
                            match fs::read(&file_path) {
                                Ok(raw_bytes) => {
                                    let header = format!(
                                        "HTTP/1.1 200 OK\r\n{}\r\nContent-Length: {}\r\n\r\n",
                                        mime,
                                        raw_bytes.len()
                                    );

                                    stream.write_all(header.as_bytes()).expect("Failed to write response header");
                                    stream.write_all(&raw_bytes).expect("Failed to write response body");
                                },
                                Err(_e) => {
                                    let error_response = String::from("HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/html\r\n\r\n<h1>404 Not Found</h1>");

                                    stream.write_all(error_response.as_bytes()).expect("Failed to write to stream");
                                }
                            }
                        } else {
                            match fs::read_to_string(&file_path) {
                                Ok(file) => {
                                    let response = format!("HTTP/1.1 200 OK\r\n{}\r\n\r\n{}", mime, &file);

                                    stream.write_all(response.as_bytes()).expect("Failed to write to stream");
                                },
                                Err(_e) => {
                                    let error_response = String::from("HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/html\r\n\r\n<h1>404 Not Found</h1>");

                                    stream.write_all(error_response.as_bytes()).expect("Failed to write to stream");
                                }
                            }
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

use std::collections::HashMap;
use std::str::from_utf8;

pub struct Request {
    pub method: String,
    pub path: String,
    pub query: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl Request {
    pub fn parser(raw_bytes: &[u8]) -> Self {

        let mut method = String::new();
        let mut path = String::new();
        let mut query = String::new();
        let mut version = String::new();
        let mut headers = HashMap::new();
        let mut body = String::new();

        let mut counter: u8 = 0;
        let mut cursor: usize = 0;

        while counter < 6 {

            // method
            if counter == 0 {
                let start = cursor;
                while raw_bytes.len() > cursor && raw_bytes[cursor] != b' ' {
                    cursor += 1;
                }

                let temp_method = from_utf8(&raw_bytes[start..cursor]).unwrap().to_string();

                // verify if the method is valid or not and generate error (to do)

                method = temp_method;
                counter += 1;
                cursor += 1;
            }

            // path
            if counter == 1 {
                let start = cursor;
                while raw_bytes.len() > cursor && raw_bytes[cursor] != b'?' && raw_bytes[cursor] != b' ' {
                    cursor += 1;
                }

                if raw_bytes[cursor] == b'?' {
                    path = from_utf8(&raw_bytes[start..cursor]).unwrap().to_string();
                    counter += 1;
                    cursor += 1;
                } else if raw_bytes[cursor] == b' ' {
                    path = from_utf8(&raw_bytes[start..cursor]).unwrap().to_string();
                    counter += 2;
                    cursor += 1;
                }
            }

            // query
            if counter == 2 {
                let start = cursor;
                while raw_bytes.len() > cursor && raw_bytes[cursor] != b' ' {
                    cursor += 1;
                }

                // extra query parsing (to do)

                query = from_utf8(&raw_bytes[start..cursor]).unwrap().to_string();
                counter += 1;
                cursor += 1;
            }

            // version
            if counter == 3 {
                let start = cursor;
                while raw_bytes.len() > cursor && raw_bytes[cursor] != b'\r' {
                    cursor += 1;
                }

                // version error (to do)

                version = from_utf8(&raw_bytes[start..cursor]).unwrap().to_string();
                counter += 1;
                cursor += 2;
            }

            // headers
            if counter == 4 {
                let start = cursor;
                cursor = raw_bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
                let temp_headers = from_utf8(&raw_bytes[start..cursor]).unwrap().to_string();

                let lines: Vec<&str> = temp_headers.split("\r\n").collect();

                for line in lines {
                    let (header_name, header_value) = line.split_once(':').unwrap();

                    let header_value = header_value.trim();

                    match header_name {
                        "Host"
                        | "User-Agent"
                        | "Accept"
                        | "Content-Type"
                        | "Content-Length"
                        | "Authorization"
                        | "Connection" => {
                            headers.insert(
                                header_name.to_string(),
                                header_value.to_string(),
                            );
                        }
                        _ => {}
                    }
                }

                counter += 2;
                cursor += 4;
            }

            // body
            if cursor == 5 && method != "GET" {
                let content_length = headers.get("Content-Length").unwrap().parse::<usize>().unwrap();

                body =  from_utf8(&raw_bytes[cursor..cursor+content_length]).unwrap().to_string();
            }

        }

        Request { method, path, query, version, headers, body }
    }
}
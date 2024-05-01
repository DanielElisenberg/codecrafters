use clap::Parser;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "")]
    directory: String,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Request {
    fn new_from_buffer(buffer: &[u8]) -> Request {
        let buffer_string = String::from_utf8_lossy(buffer);
        let start_line = buffer_string.lines().next().unwrap();
        let mut start_line_parts = start_line.split_whitespace();
        let method = start_line_parts.next().unwrap();
        let path = start_line_parts.next().unwrap();

        let mut headers = HashMap::new();
        for line in buffer_string
            .split("\r\n\r\n")
            .next()
            .unwrap()
            .lines()
            .skip(1)
        {
            if line.is_empty() {
                break;
            }
            let mut parts = line.splitn(2, ": ");
            let key = parts.next().unwrap();
            let value = parts.next().unwrap();
            headers.insert(key.to_string(), value.to_string());
        }

        Request {
            method: method.to_string(),
            path: path.to_string(),
            version: headers
                .get("HTTP/1.1")
                .unwrap_or(&"HTTP/1.1".to_string())
                .to_string(),
            headers,
            body: buffer_string
                .split("\r\n\r\n")
                .nth(1)
                .unwrap_or(&"".to_string())
                .to_string(),
        }
    }
}

fn connection_handler(mut stream: TcpStream, config: Args) {
    let buffer = &mut [0u8; 1024];
    stream.read(buffer).unwrap();
    let request = Request::new_from_buffer(buffer);
    match request.path.as_str() {
        "/" => {
            stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        }
        path if path.starts_with("/echo") => {
            let echo_path = path.strip_prefix("/echo/").unwrap();
            stream
                .write(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        echo_path.len(),
                        echo_path
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
        path if path.starts_with("/user-agent") => {
            stream
                .write(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        request.headers.get("User-Agent").unwrap().len(),
                        request.headers.get("User-Agent").unwrap()
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
        path if path.starts_with("/files") => match request.method.as_str() {
            "GET" => {
                let file_path = config.directory + path.strip_prefix("/files/").unwrap();
                if !std::path::Path::new(&file_path).exists() {
                    stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                    return;
                }
                let contents = std::fs::read_to_string(file_path).unwrap();
                stream.write(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                        contents.len(),
                        contents
                    ).as_bytes(),
                ).unwrap();
            }
            "POST" => {
                let file_path = config.directory + path.strip_prefix("/files/").unwrap();
                let contents = request.body.trim_end_matches('\0').to_string();
                std::fs::write(file_path, contents).unwrap();
                stream.write("HTTP/1.1 201 OK\r\n\r\n".as_bytes()).unwrap();
            }
            _ => {
                stream
                    .write(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n")
                    .unwrap();
                return;
            }
        },
        _ => {
            stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
        }
    }
    stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
    stream.flush().unwrap();
}

fn main() {
    let config = Args::parse();
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let conf_clone = config.clone();
                thread::spawn(move || {
                    connection_handler(stream, conf_clone);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

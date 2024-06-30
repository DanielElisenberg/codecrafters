mod adapters;
mod models;

use crate::adapters::local_storage;
use crate::models::{Request, Response, StatusCode};

use clap::Parser;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "")]
    directory: String,
}

fn handle_root(mut stream: &TcpStream) {
    stream
        .write(
            &Response::new(
                StatusCode::OK,
                Some(vec![("Content-Encoding", "gzip")]),
                None,
            )
            .to_bytes(),
        )
        .unwrap();
}

fn handle_echo_path(mut stream: &TcpStream, echo_path: &str, compress: bool) {
    let echo_path_len = echo_path.len().to_string();
    let mut response_headers = vec![
        ("Content-Type", "text/plain"),
        ("Content-Length", &echo_path_len),
    ];
    if compress {
        response_headers.push(("Content-Encoding", "gzip"));
    }
    stream
        .write(
            &Response::new(
                StatusCode::OK,
                Some(response_headers),
                Some(echo_path.to_string()),
            )
            .to_bytes(),
        )
        .unwrap();
}

fn handle_echo_user_agent(mut stream: &TcpStream, user_agent: &str) {
    stream
        .write(
            &Response::new(
                StatusCode::OK,
                Some(vec![
                    ("Content-Type", "text/plain"),
                    ("Content-Length", &format!("{}", user_agent.len())),
                ]),
                Some(user_agent.to_string()),
            )
            .to_bytes(),
        )
        .unwrap();
}

fn handle_get_file(mut stream: &TcpStream, file_path: String) {
    match local_storage::read_file_to_string(file_path) {
        Ok(file_content) => {
            stream
                .write(
                    &Response::new(
                        StatusCode::OK,
                        Some(vec![
                            ("Content-Type", "application/octet-stream"),
                            ("Content-Length", &file_content.len().to_string()),
                        ]),
                        Some(file_content),
                    )
                    .to_bytes(),
                )
                .unwrap();
        }
        Err(_) => {
            stream
                .write(&Response::new_from_status_code(StatusCode::NotFound).to_bytes())
                .unwrap();
        }
    }
}

fn handle_post_file(mut stream: &TcpStream, file_path: String, contents: String) {
    match local_storage::write_file_to_string(file_path, contents) {
        Ok(_) => {
            stream
                .write(&Response::new_from_status_code(StatusCode::Created).to_bytes())
                .unwrap();
        }
        Err(_) => {
            stream
                .write(&Response::new_from_status_code(StatusCode::InternalServerError).to_bytes())
                .unwrap();
        }
    }
}

fn connection_handler(mut stream: TcpStream, config: Args) {
    let buffer = &mut [0u8; 1024];
    stream.read(buffer).unwrap();
    let request = Request::new_from_buffer(buffer);
    match request.path.as_str() {
        "/" => handle_root(&stream),
        path if path.starts_with("/echo") => handle_echo_path(
            &stream,
            path.strip_prefix("/echo/").unwrap(),
            request.has_content_encoding_gzip(),
        ),
        path if path.starts_with("/user-agent") => {
            handle_echo_user_agent(&stream, request.headers.get("User-Agent").unwrap())
        }
        path if path.starts_with("/files") => match request.method.as_str() {
            "GET" => handle_get_file(
                &stream,
                config.directory + path.strip_prefix("/files/").unwrap(),
            ),
            "POST" => handle_post_file(
                &stream,
                config.directory + path.strip_prefix("/files/").unwrap(),
                request.body.trim_end_matches('\0').to_string(),
            ),
            _ => {
                stream
                    .write(&Response::new_from_status_code(StatusCode::MethodNotAllowed).to_bytes())
                    .unwrap();
                return;
            }
        },
        _ => {
            stream
                .write(&Response::new_from_status_code(StatusCode::NotFound).to_bytes())
                .unwrap();
        }
    }
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

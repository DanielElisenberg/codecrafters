use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::Write;

#[allow(dead_code)]
#[derive(Debug)]
pub enum StatusCode {
    OK = 200,
    Created = 201,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
    MethodNotAllowed = 405,
}

impl StatusCode {
    pub fn to_string(&self) -> String {
        match self {
            StatusCode::OK => "200 OK".to_string(),
            StatusCode::Created => "201 Created".to_string(),
            StatusCode::BadRequest => "400 Bad Request".to_string(),
            StatusCode::NotFound => "404 Not Found".to_string(),
            StatusCode::InternalServerError => "500 Internal Server Error".to_string(),
            StatusCode::MethodNotAllowed => "405 Method Not Allowed".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Response {
    version: String,
    status_code: StatusCode,
    headers: HashMap<String, String>,
    body: String,
}

impl Response {
    pub fn new(
        status_code: StatusCode,
        headers: Option<Vec<(&str, &str)>>,
        body: Option<String>,
    ) -> Response {
        let mut header_map = HashMap::new();
        for (key, value) in headers.unwrap_or(vec![]) {
            header_map.insert(key.to_string(), value.to_string());
        }
        Response {
            version: "HTTP/1.1".to_string(),
            status_code,
            headers: header_map,
            body: body.unwrap_or("".to_string()),
        }
    }

    pub fn new_from_status_code(status_code: StatusCode) -> Response {
        Response {
            version: "HTTP/1.1".to_string(),
            status_code,
            headers: HashMap::new(),
            body: "".to_string(),
        }
    }

    pub fn has_content_encoding_gzip(&self) -> bool {
        self.headers
            .get("Content-Encoding")
            .unwrap_or(&"".to_string())
            == "gzip"
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = String::new();
        response.push_str(&self.version);
        response.push_str(" ");
        response.push_str(&self.status_code.to_string());
        response.push_str("\r\n");

        if self.has_content_encoding_gzip() {
            let mut compressor = GzEncoder::new(Vec::new(), Compression::default());
            let _ = compressor.write_all(&self.body.clone().into_bytes());
            let compressed_body = compressor.finish().unwrap();
            for (key, value) in self.headers.clone() {
                if key == "Content-Length" {
                    response.push_str(&format!(
                        "Content-Length: {}\r\n",
                        compressed_body.len().to_string()
                    ));
                } else {
                    response.push_str(&key);
                    response.push_str(": ");
                    response.push_str(&value);
                    response.push_str("\r\n");
                }
            }
            response.push_str("\r\n");
            let mut response_bytes = response.into_bytes();
            response_bytes.extend(compressed_body);
            response_bytes
        } else {
            for (key, value) in self.headers.clone() {
                response.push_str(&key);
                response.push_str(": ");
                response.push_str(&value);
                response.push_str("\r\n");
            }
            response.push_str("\r\n");
            response.push_str(&self.body);
            response.into_bytes()
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Request {
    pub fn new_from_buffer(buffer: &[u8]) -> Request {
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
    pub fn has_content_encoding_gzip(&self) -> bool {
        self.headers
            .get("Accept-Encoding")
            .unwrap_or(&"".to_string())
            .split(", ")
            .collect::<Vec<&str>>()
            .into_iter()
            .any(|value| value == "gzip")
    }
}

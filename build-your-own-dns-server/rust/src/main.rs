mod models;
mod traits;

use crate::traits::Encodable;
use clap::Parser;
use models::{DnsAnswer, DnsHeader, DnsPacket};
use std::net::UdpSocket;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "")]
    resolver: String,
}

// Domain name specification: https://www.rfc-editor.org/rfc/rfc1035

#[allow(dead_code)]
fn generate_response(dns_request: DnsPacket) -> DnsPacket {
    return DnsPacket {
        dns_header: DnsHeader::from_request_header(dns_request.dns_header),
        dns_questions: dns_request.dns_questions.clone(),
        dns_answers: dns_request
            .dns_questions
            .iter()
            .map(|dns_question| DnsAnswer::from_request_question(&dns_question))
            .collect(),
    };
}

fn resolve_response_upstream(
    udp_socket: &UdpSocket,
    upstream_addr: &String,
    dns_request: DnsPacket,
) -> DnsPacket {
    let upstream_requests = dns_request.split();
    let upstream_replies = upstream_requests
        .into_iter()
        .map(|upstream_request| {
            udp_socket
                .send_to(&upstream_request.encode(), upstream_addr)
                .expect("Failed to send request upstream");
            let mut forward_buf = [0; 512];
            udp_socket
                .recv_from(&mut forward_buf)
                .expect("Failed to receive response from upstream");
            return DnsPacket::decode(forward_buf);
        })
        .collect();
    return DnsPacket::merge(upstream_replies);
}

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let config = Args::parse();
    let mut buf = [0; 512];
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let dns_request = DnsPacket::decode(buf);
                // let dns_response = generate_response(dns_request);
                let dns_response =
                    resolve_response_upstream(&udp_socket, &config.resolver, dns_request);
                udp_socket
                    .send_to(&dns_response.encode(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}

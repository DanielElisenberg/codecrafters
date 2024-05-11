use crate::models::{DnsAnswer, DnsHeader, DnsQuestion};
use crate::traits::{Decodable, Encodable};

#[derive(Debug)]
pub struct DnsPacket {
    pub dns_header: DnsHeader,
    pub dns_questions: Vec<DnsQuestion>,
    pub dns_answers: Vec<DnsAnswer>,
}

impl DnsPacket {
    pub fn decode(buffer: [u8; 512]) -> DnsPacket {
        let dns_header =
            DnsHeader::decode(buffer.to_vec()).expect("Error when building DnsRequest from bytes");
        let answer_count = dns_header.answer_record_count as usize;
        let question_count = dns_header.question_count as usize;
        let mut cursor = 12;
        return DnsPacket {
            dns_header,
            dns_questions: (0..question_count)
                .map(|_index| {
                    DnsQuestion::decode_with_cursor(buffer.to_vec(), &mut cursor)
                        .expect("Error when building DnsQuestion from bytes")
                })
                .collect(),
            dns_answers: (0..answer_count)
                .map(|_index| {
                    DnsAnswer::decode_with_cursor(buffer.to_vec(), &mut cursor)
                        .expect("Error when building DnsAnswer from bytes")
                })
                .collect(),
        };
    }

    pub fn split(&self) -> Vec<DnsPacket> {
        let mut dns_header = self.dns_header.clone();
        dns_header.question_count = 1;
        return self
            .dns_questions
            .clone()
            .into_iter()
            .map(|dns_question| DnsPacket {
                dns_header: dns_header.clone(),
                dns_questions: vec![dns_question],
                dns_answers: vec![],
            })
            .collect();
    }

    pub fn merge(dns_packets: Vec<DnsPacket>) -> DnsPacket {
        let mut dns_header = dns_packets[0].dns_header.clone();
        dns_header.question_count = dns_packets.len() as u16;
        dns_header.answer_record_count = dns_packets.len() as u16;
        let mut dns_questions: Vec<DnsQuestion> = vec![];
        let mut dns_answers: Vec<DnsAnswer> = vec![];
        dns_packets.into_iter().for_each(|dns_packet| {
            dns_questions.extend(dns_packet.dns_questions);
            dns_answers.extend(dns_packet.dns_answers);
        });

        return DnsPacket {
            dns_header,
            dns_questions,
            dns_answers,
        };
    }
}

impl Encodable for DnsPacket {
    fn encode(&self) -> Vec<u8> {
        let mut encoded_dns_request: Vec<u8> = self.dns_header.encode();
        for dns_question in self.dns_questions.iter() {
            encoded_dns_request.extend(dns_question.encode());
        }
        for dns_answer in self.dns_answers.iter() {
            encoded_dns_request.extend(dns_answer.encode());
        }
        encoded_dns_request
    }
}

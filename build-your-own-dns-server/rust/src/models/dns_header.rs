use crate::traits::{Decodable, Encodable};

#[derive(Debug, Clone)]
enum QueryResponse {
    ReplyPacket = 1,
    QuestionPacket = 2,
}

#[derive(Debug, Clone)]
pub struct DnsHeader {
    packet_identifier: u16,
    query_response_indicator: QueryResponse,
    operation_code: u8,
    authoritative_answer: bool,
    truncation: bool,
    recursion_desired: bool,
    recursion_available: bool,
    reserved: u8,
    response_code: u8,
    pub question_count: u16,
    pub answer_record_count: u16,
    authority_record_count: u16,
    additional_record_count: u16,
}

impl DnsHeader {
    pub fn from_request_header(request_header: DnsHeader) -> DnsHeader {
        return DnsHeader {
            packet_identifier: request_header.packet_identifier,
            query_response_indicator: QueryResponse::ReplyPacket,
            operation_code: request_header.operation_code,
            authoritative_answer: false,
            truncation: false,
            recursion_desired: request_header.recursion_desired,
            recursion_available: false,
            reserved: 0,
            response_code: match request_header.operation_code {
                0 => 0,
                _ => 4,
            },
            question_count: request_header.question_count,
            answer_record_count: request_header.question_count,
            authority_record_count: 0,
            additional_record_count: 0,
        };
    }
}

impl Decodable for DnsHeader {
    fn decode_with_cursor(buffer: Vec<u8>, _cursor: &mut usize) -> Result<Self, String> {
        Self::decode(buffer)
    }
    fn decode(buffer: Vec<u8>) -> Result<DnsHeader, String> {
        return Ok(DnsHeader {
            packet_identifier: (buffer[0] as u16) << 8 | buffer[1] as u16,
            query_response_indicator: match buffer[2] >> 7 {
                1 => QueryResponse::ReplyPacket,
                0 => QueryResponse::QuestionPacket,
                _ => {
                    return Err(
                        "Error when converting buffer to query response indicator".to_string()
                    )
                }
            },
            operation_code: buffer[2] >> 3 & 0b1111,
            authoritative_answer: match buffer[2] >> 2 & 0b1 {
                1 => true,
                0 => false,
                _ => return Err("Error when converting buffer to operation code".to_string()),
            },
            truncation: match buffer[2] >> 1 & 0b1 {
                1 => true,
                0 => false,
                _ => return Err("Error when converting buffer to truncation".to_string()),
            },
            recursion_desired: match buffer[2] & 0b1 {
                1 => true,
                0 => false,
                _ => return Err("Error when converting buffer to recursion desired".to_string()),
            },
            recursion_available: match buffer[3] >> 7 & 0b1 {
                1 => true,
                0 => false,
                _ => return Err("Error when converting buffer to recursion available".to_string()),
            },
            reserved: buffer[3] >> 4 & 0b111,
            response_code: buffer[3] & 0b1111,
            question_count: (buffer[4] as u16) << 8 | buffer[5] as u16,
            answer_record_count: (buffer[6] as u16) << 8 | buffer[7] as u16,
            authority_record_count: (buffer[8] as u16) << 8 | buffer[9] as u16,
            additional_record_count: (buffer[10] as u16) << 8 | buffer[11] as u16,
        });
    }
}

impl Encodable for DnsHeader {
    fn encode(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];

        buffer.extend_from_slice(&self.packet_identifier.to_be_bytes());
        let qr = match self.query_response_indicator {
            QueryResponse::ReplyPacket => 1 as u8,
            QueryResponse::QuestionPacket => 0 as u8,
        } << 7;
        let op_code = (self.operation_code & 0b1111) << 3;
        let aa = (self.authoritative_answer as u8) << 2;
        let tc = (self.truncation as u8) << 1;
        let rd = self.recursion_desired as u8;
        buffer.push(qr | op_code | aa | tc | rd);

        let ra = (self.recursion_available as u8) << 7;
        let z = (self.reserved & 0b111) << 4;
        let rcode = self.response_code & 0b1111;
        buffer.push(ra | z | rcode);

        buffer.extend_from_slice(&self.question_count.to_be_bytes());
        buffer.extend_from_slice(&self.answer_record_count.to_be_bytes());
        buffer.extend_from_slice(&self.authority_record_count.to_be_bytes());
        buffer.extend_from_slice(&self.additional_record_count.to_be_bytes());

        buffer
    }
}

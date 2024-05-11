use crate::models::{Class, Label, RecordType};
use crate::traits::{Decodable, Encodable};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DnsQuestion {
    pub record_type: RecordType,
    pub class: Class,
    pub labels: Vec<Label>,
}

impl Decodable for DnsQuestion {
    fn decode(buffer: Vec<u8>) -> Result<Self, String> {
        Self::decode_with_cursor(buffer, &mut 0)
    }
    fn decode_with_cursor(buffer: Vec<u8>, cursor: &mut usize) -> Result<DnsQuestion, String> {
        let mut labels: Vec<Label> = vec![];
        let mut pointer_cursor: usize = *cursor;
        let mut is_pointer: bool = false;

        while buffer[pointer_cursor] != 0x00 {
            // If the two most significant bits are 11 this and the next byte
            // (Sans the two most significant bits) contain a reference
            // to an earlier label for compression.
            // Source: https://www.rfc-editor.org/rfc/rfc1035#section-4.1.4
            is_pointer = (buffer[pointer_cursor] & 0b11000000) == 0b11000000;
            if is_pointer {
                pointer_cursor = ((((buffer[pointer_cursor] & 0b00111111) as u16) << 8)
                    | (buffer[pointer_cursor + 1] as u16))
                    as usize;
                *cursor += 3;
            }
            let length = buffer[pointer_cursor] as usize;
            pointer_cursor += 1;
            let label_bytes = &buffer[pointer_cursor..pointer_cursor + length];
            pointer_cursor += length;
            let label_string = std::str::from_utf8(label_bytes).unwrap();
            labels.push(Label::from_string(label_string.to_string()))
        }
        if !is_pointer {
            *cursor = pointer_cursor + 1;
        }
        *cursor += 4;
        return Ok(DnsQuestion {
            labels,
            record_type: RecordType::decode(buffer[*cursor - 4..*cursor - 2].to_vec()).unwrap(),
            class: Class::decode(buffer[*cursor - 2..*cursor].to_vec()).unwrap(),
        });
    }
}

impl Encodable for DnsQuestion {
    fn encode(&self) -> Vec<u8> {
        let mut encoded_dns_question: Vec<u8> = self
            .labels
            .iter()
            .flat_map(|label| label.encode())
            .collect();
        encoded_dns_question.extend(vec![0x00]);
        encoded_dns_question.extend(vec![0, 1, 0, 1]);
        encoded_dns_question
    }
}

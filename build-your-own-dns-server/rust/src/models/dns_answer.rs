use crate::models::{Class, DnsQuestion, Label, RecordType};
use crate::traits::{Decodable, Encodable};

#[allow(dead_code)]
#[derive(Debug)]
pub struct DnsAnswer {
    name: Vec<Label>,
    record_type: RecordType,
    class: Class,
    time_to_live: u32,
    length: u16,
    data: Vec<u8>,
}

impl DnsAnswer {
    pub fn from_request_question(request_question: &DnsQuestion) -> DnsAnswer {
        DnsAnswer {
            name: request_question.labels.clone(),
            record_type: RecordType::A,
            class: Class::IN,
            time_to_live: 60,
            length: 4,
            data: vec![8, 8, 8, 8],
        }
    }
}

impl Decodable for DnsAnswer {
    fn decode(buffer: Vec<u8>) -> Result<DnsAnswer, String> {
        Self::decode_with_cursor(buffer, &mut 0)
    }

    fn decode_with_cursor(buffer: Vec<u8>, cursor: &mut usize) -> Result<DnsAnswer, String> {
        let mut labels: Vec<Label> = vec![];
        let mut pointer_cursor: usize = *cursor;
        let mut is_pointer: bool = false;

        while buffer[pointer_cursor] != 0x00 {
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
        let record_type = RecordType::decode(buffer[*cursor - 4..*cursor - 2].to_vec()).unwrap();
        let class = Class::decode(buffer[*cursor - 2..*cursor].to_vec()).unwrap();
        let time_to_live = (buffer[*cursor] as u32) << 24
            | (buffer[*cursor + 1] as u32) << 16
            | (buffer[*cursor + 2] as u32) << 8
            | (buffer[*cursor + 3] as u32);
        *cursor += 4;
        let length = (buffer[*cursor] as u16) << 8 | buffer[*cursor + 1] as u16;
        let data = vec![
            buffer[*cursor + 2],
            buffer[*cursor + 3],
            buffer[*cursor + 4],
            buffer[*cursor + 5],
        ];
        *cursor += 2 + length as usize;
        return Ok(DnsAnswer {
            name: labels,
            record_type,
            class,
            time_to_live,
            length,
            data,
        });
    }
}

impl Encodable for DnsAnswer {
    fn encode(&self) -> Vec<u8> {
        let mut encoded_dns_answer: Vec<u8> =
            self.name.iter().flat_map(|label| label.encode()).collect();
        encoded_dns_answer.extend(vec![0x00]);
        encoded_dns_answer.extend(vec![0, 1, 0, 1]);
        encoded_dns_answer.extend(Vec::from(self.time_to_live.to_be_bytes()));
        encoded_dns_answer.extend(Vec::from(self.length.to_be_bytes()));
        encoded_dns_answer.extend(self.data.clone());
        encoded_dns_answer
    }
}

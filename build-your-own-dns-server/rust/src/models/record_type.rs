use crate::traits::Decodable;

// specification: https://www.rfc-editor.org/rfc/rfc1035#section-3.2.2
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum RecordType {
    A = 1,      // a host address
    NS = 2,     // an authoritative name server
    MD = 3,     // a mail destination (Obsolete - use MX)
    MF = 4,     // a mail forwarder (Obsolete - use MX)
    CNAME = 5,  // the canonical name for an alias
    SOA = 6,    // marks the start of a zone of authority
    MB = 7,     // a mailbox domain name (EXPERIMENTAL)
    MG = 8,     // a mail group member (EXPERIMENTAL)
    MR = 9,     // a mail rename domain name (EXPERIMENTAL)
    NULL = 10,  // a null RR (EXPERIMENTAL)
    WKS = 11,   // a well known service description
    PTR = 12,   // a domain name pointer
    HINFO = 13, // host information
    MINFO = 14, // mailbox or mail list information
    MX = 15,    // mail exchange
    TXT = 16,   // text strings}
}

impl Decodable for RecordType {
    fn decode(buffer: Vec<u8>) -> Result<RecordType, String> {
        let u16_value = ((buffer[0] as u16) << 8) | (buffer[1] as u16);
        match u16_value {
            1 => Ok(RecordType::A),
            2 => Ok(RecordType::NS),
            5 => Ok(RecordType::CNAME),
            6 => Ok(RecordType::SOA),
            7 => Ok(RecordType::MB),
            8 => Ok(RecordType::MG),
            9 => Ok(RecordType::MR),
            10 => Ok(RecordType::NULL),
            11 => Ok(RecordType::WKS),
            12 => Ok(RecordType::PTR),
            13 => Ok(RecordType::HINFO),
            14 => Ok(RecordType::HINFO),
            15 => Ok(RecordType::MX),
            16 => Ok(RecordType::TXT),
            _ => Err("Could not decode RecordType from invalid value {:u16_value}".to_string()),
        }
    }
    fn decode_with_cursor(buffer: Vec<u8>, cursor: &mut usize) -> Result<Self, String> {
        Self::decode(buffer[*cursor..].to_vec())
    }
}

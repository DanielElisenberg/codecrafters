use crate::traits::Decodable;

// specification: https://www.rfc-editor.org/rfc/rfc1035#section-3.2.4
#[derive(Debug, Clone)]
pub enum Class {
    IN = 1, // the Internet
    CS = 2, // the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    CH = 3, // the CHAOS class
    HS = 4, // Hesiod [Dyer 87]
}

impl Decodable for Class {
    fn decode(buffer: Vec<u8>) -> Result<Class, String> {
        let u16_value = ((buffer[0] as u16) << 8) | (buffer[1] as u16);
        match u16_value {
            1 => Ok(Class::IN),
            2 => Ok(Class::CS),
            3 => Ok(Class::CH),
            4 => Ok(Class::HS),
            _ => Err("Could not decode RecordType from invalid value {:u16_value}".to_string()),
        }
    }
    fn decode_with_cursor(buffer: Vec<u8>, cursor: &mut usize) -> Result<Self, String> {
        Self::decode(buffer[*cursor..].to_vec())
    }
}

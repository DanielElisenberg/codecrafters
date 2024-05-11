pub trait Encodable {
    fn encode(&self) -> Vec<u8>;
}

pub trait Decodable {
    fn decode(buffer: Vec<u8>) -> Result<Self, String>
    where
        Self: Sized;

    fn decode_with_cursor(buffer: Vec<u8>, cursor: &mut usize) -> Result<Self, String>
    where
        Self: Sized;
}

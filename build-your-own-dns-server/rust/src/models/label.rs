use crate::traits::Encodable;

#[derive(Clone, Debug)]
pub struct Label {
    length: u8,
    content: String,
}

impl Label {
    pub fn from_string(string: String) -> Label {
        Label {
            length: string.len() as u8,
            content: string,
        }
    }
}
impl Encodable for Label {
    fn encode(&self) -> Vec<u8> {
        let mut encoded_label = vec![self.length];
        encoded_label.extend(
            self.content
                .chars()
                .flat_map(|c| c.to_string().into_bytes())
                .collect::<Vec<u8>>(),
        );
        encoded_label
    }
}

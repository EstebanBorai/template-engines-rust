use serde_derive::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Person {
    pub id: u32,
    pub name: String,
}

impl Person {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}

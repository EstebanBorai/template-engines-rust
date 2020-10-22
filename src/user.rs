use serde_derive::Serialize;

#[derive(Serialize, Clone, Copy, PartialEq, Debug)]
pub enum DbPrivilege {
    CanRead,
    CanWrite,
}

#[derive(Serialize, Clone, Debug)]
pub struct User {
    pub username: String,
    pub password: String,
    pub privileges: Vec<DbPrivilege>,
}

impl User {
    pub fn new(username: &str, password: &str, privileges: Vec<DbPrivilege>) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            privileges,
        }
    }
}

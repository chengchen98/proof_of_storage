pub struct Message {
    pub size: usize
}

pub struct CryptoBulletin {
    pub state: u8,
    pub price: u32,
    pub lifetime: u32,
    pub info: &str,
    pub message: Message,
    pub signature: &str
}
pub struct Message {
    pub size: usize
}

pub struct CryptoBulletin<'a> {
    pub state: u8, // CB的状态
    pub price: u32, // CB的价值
    pub lifetime: u32,
    pub info: &'a str,
    pub message: Message,
    pub signature: &'a str
}

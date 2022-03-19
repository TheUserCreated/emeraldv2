#[derive(Debug)]
pub struct Greeting {
    pub greeting_text: String,
    pub channel_id_internal: u64,
    pub role_id_internal: u64,
}

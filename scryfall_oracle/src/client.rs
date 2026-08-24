use reqwest::{
    Client,
    header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT},
};

const USER_AGENT_VALUE: &str = "momir_rs/0.1.0";

#[derive(Debug, Clone)]
pub struct ScryfallClient {
    pub(crate) client: Client,
}

impl ScryfallClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        let mut headers = HeaderMap::new();

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client })
    }
    // TODO Add rate limit
}

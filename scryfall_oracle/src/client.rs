use reqwest::{
    Client,
    header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

const USER_AGENT_VALUE: &str = "momir_rs/0.1.0";
const REQUEST_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub struct ScryfallClient {
    client: Client,
    last_request: Mutex<Instant>,
}

impl ScryfallClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        let mut headers = HeaderMap::new();

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            last_request: Mutex::new(Instant::now() - REQUEST_INTERVAL),
        })
    }

    pub async fn get<U>(
        &self,
        url: U,
        query: Option<&HashMap<&str, &str>>,
    ) -> Result<reqwest::Response, reqwest::Error>
    where
        U: reqwest::IntoUrl,
    {
        let mut last_request = self.last_request.lock().await;

        let elapsed = last_request.elapsed();

        if elapsed < REQUEST_INTERVAL {
            tokio::time::sleep(REQUEST_INTERVAL - elapsed).await;
        }

        *last_request = Instant::now();

        drop(last_request);

        let mut request = self.client.get(url);

        if let Some(query) = query {
            request = request.query(query);
        }

        request.send().await
    }
}

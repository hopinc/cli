use crate::config::{HOP_API_BASE_URL, PLATFORM, VERSION};
use reqwest::blocking::Client as BlockingClient;
use reqwest::header::HeaderMap;
use reqwest::Client as AsyncClient;

#[derive(Debug, Clone)]
pub struct HttpClient {
    pub client: AsyncClient,
    pub base_url: String,
    pub headers: HeaderMap,
    pub ua: String,
}

impl HttpClient {
    pub fn sync_client(self) -> BlockingClient {
        BlockingClient::builder()
            .user_agent(self.ua)
            .default_headers(self.headers)
            .build()
            .unwrap()
    }

    pub fn new(token: Option<String>) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert("content-type", "application/json".parse().unwrap());

        if token.is_some() {
            headers.insert("Authorization", token.clone().unwrap().parse().unwrap());
        }

        let ua = format!("hop_cli/{} on {}", VERSION, PLATFORM);

        Self {
            headers: headers.clone(),
            ua: ua.clone(),
            client: AsyncClient::builder()
                .user_agent(ua.clone())
                .default_headers(headers.clone())
                .build()
                .unwrap(),
            base_url: HOP_API_BASE_URL.to_string(),
        }
    }

    pub async fn request<T>(
        &self,
        method: &str,
        path: &str,
        body: Option<String>,
    ) -> Result<Option<T>, reqwest::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut request = self.client.request(
            method.parse().unwrap(),
            &format!("{}{}", self.base_url, path),
        );

        if let Some(body) = body {
            request = request.body(body);
        }

        let response = request
            .send()
            .await
            .expect("Failed to send request")
            .json::<T>()
            .await;

        match response {
            Ok(response) => Ok(Some(response)),
            Err(_) => Ok(None),
        }
    }
}

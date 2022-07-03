use crate::config::{HOP_API_BASE_URL, PLATFORM, VERSION};
use crate::types::ErrorResponse;
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
    pub fn new(token: Option<String>) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert("content-type", "application/json".parse().unwrap());

        if let Some(token) = token {
            headers.insert("Authorization", token.parse().unwrap());
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
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let mut request = self.client.request(
            method.parse().unwrap(),
            &format!("{}{}", self.base_url, path),
        );

        if let Some(body) = body {
            request = request.body(body);
        }

        let response = request.send().await.expect("Failed to send request");

        let response = match response.status() {
            reqwest::StatusCode::OK => response,
            reqwest::StatusCode::NO_CONTENT => return Ok(None),
            _ => {
                let body = response.json::<ErrorResponse>().await;

                match body {
                    Ok(body) => {
                        panic!("{}", body.error.message)
                    }
                    Err(err) => {
                        panic!("{}", err)
                    }
                }
            }
        };

        let response = response
            .json::<T>()
            .await
            .expect("Failed to parse response");

        Ok(Some(response))
    }
}

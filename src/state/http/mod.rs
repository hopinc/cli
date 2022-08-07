mod types;

use anyhow::{anyhow, Result};
use reqwest::header::HeaderMap;
use reqwest::Client as AsyncClient;

use self::types::{Base, ErrorResponse};
use crate::config::VERSION;

const HOP_API_BASE_URL: &str = "https://api.hop.io/v1";

#[derive(Debug, Clone)]
pub struct HttpClient {
    pub client: AsyncClient,
    pub base_url: String,
    pub headers: HeaderMap,
    pub ua: String,
}

impl HttpClient {
    pub fn new(token: Option<String>, api_url: Option<String>) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert("accept", "application/json".parse().unwrap());

        if let Some(token) = token {
            headers.insert("authorization", token.parse().unwrap());
        }

        let ua = format!(
            "hop_cli/{} on {}",
            VERSION,
            sys_info::os_type().unwrap_or_else(|_| "unknown".to_string())
        );

        let base_url = match api_url {
            Some(url) => url,
            None => HOP_API_BASE_URL.to_string(),
        };

        Self {
            client: AsyncClient::builder()
                .user_agent(ua.clone())
                .default_headers(headers.clone())
                .build()
                .unwrap(),
            base_url,
            headers,
            ua,
        }
    }

    pub async fn handle_response<T>(&self, response: reqwest::Response) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = match response.status() {
            reqwest::StatusCode::OK => response,
            reqwest::StatusCode::CREATED => return Ok(None),
            reqwest::StatusCode::NO_CONTENT => return Ok(None),
            code => return self.handle_error(response, code.as_u16()).await,
        };

        let response = response
            .json::<Base<T>>()
            .await
            .expect("Failed to parse response");

        Ok(Some(response.data))
    }

    async fn handle_error<T>(&self, response: reqwest::Response, code: u16) -> Result<Option<T>> {
        let body = response.json::<ErrorResponse>().await;

        match body {
            Ok(body) => Err(anyhow!("{}: {}", code, body.error.message)),
            Err(err) => Err(anyhow!("Failed to parse error response: {}", err)),
        }
    }

    pub async fn request<T>(
        &self,
        method: &str,
        path: &str,
        data: Option<(reqwest::Body, &str)>,
    ) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut request = self.client.request(
            method.parse().unwrap(),
            &format!("{}{}", self.base_url, path),
        );

        log::debug!("request: {} {} {:?}", method, path, data);

        if let Some((body, content_type)) = data {
            request = request.header("content-type", content_type);
            request = request.body(body);
        }

        let request = request.build().unwrap();

        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| e.to_string())
            .expect("Failed to send the request");

        self.handle_response(response).await
    }
}

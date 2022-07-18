use reqwest::header::HeaderMap;
use reqwest::Client as AsyncClient;

use crate::config::{HOP_API_BASE_URL, PLATFORM, VERSION};
use crate::types::{Base, ErrorResponse};

/// Request data for the API
/// body, content_type
type RequestData<'a> = (hyper::Body, &'a str);

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

        headers.insert("Accept", "application/json".parse().unwrap());

        if let Some(token) = token {
            headers.insert("Authorization", token.parse().unwrap());
        }

        let ua = format!("hop_cli/{} on {}", VERSION, PLATFORM);

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

    pub async fn handle_response<T>(&self, response: reqwest::Response) -> Result<Option<T>, String>
    where
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let response = match response.status() {
            reqwest::StatusCode::OK => response,
            reqwest::StatusCode::CREATED => return Ok(None),
            reqwest::StatusCode::NO_CONTENT => return Ok(None),
            code => {
                let body = response.json::<ErrorResponse>().await;

                match body {
                    Ok(body) => return Err(format!("{}: {}", code.as_u16(), body.error.message)),
                    Err(err) => {
                        panic!("{}", err)
                    }
                };
            }
        };

        let response = response
            .json::<Base<T>>()
            .await
            .expect("Failed to parse response");

        Ok(Some(response.data))
    }

    pub async fn request<T>(
        &self,
        method: &str,
        path: &str,
        data: Option<RequestData<'_>>,
    ) -> Result<Option<T>, String>
    where
        T: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let mut request = self.client.request(
            method.parse().unwrap(),
            &format!("{}{}", self.base_url, path),
        );

        if let Some((body, content_type)) = data {
            request = request.body(body);
            request = request.header("Content-Type", content_type);
        }

        let response = request
            .send()
            .await
            .map_err(|e| e.to_string())
            .expect("Failed to send the request");

        self.handle_response(response).await
    }
}

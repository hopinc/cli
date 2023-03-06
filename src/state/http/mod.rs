mod types;

use anyhow::{anyhow, Result};
use hyper::StatusCode;
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
            "hop_cli/{VERSION} on {}",
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
            StatusCode::CREATED => return Ok(None),
            StatusCode::NO_CONTENT => return Ok(None),
            status => {
                if !status.clone().is_success() {
                    return self.handle_error(response, status).await;
                }

                response
            }
        };

        response
            .json::<Base<T>>()
            .await
            .map(|base| Some(base.data))
            .map_err(|e| anyhow!(e))
    }

    async fn handle_error<T>(
        &self,
        response: reqwest::Response,
        status: StatusCode,
    ) -> Result<Option<T>> {
        let body = response.json::<ErrorResponse>().await;

        match body {
            Ok(body) => Err(anyhow!("{}", body.error.message)),
            Err(err) => {
                log::debug!("Error deserialize message: {:#?}", err);

                Err(anyhow!("Error: HTTP {:#?}", status))
            }
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
            format!("{}{}", self.base_url, path),
        );

        log::debug!("request: {} {} {:?}", method, path, data);

        if let Some((body, content_type)) = data {
            request = request.header("content-type", content_type);

            // show body in debug mode / when developing
            #[cfg(debug_assertions)]
            log::debug!(
                "request body: {:?}",
                String::from_utf8(body.as_bytes().unwrap().to_vec())?
            );

            request = request.body(body);
        }

        let request = request.build()?;

        #[cfg(debug_assertions)]
        let now = tokio::time::Instant::now();

        let response = self.client.execute(request).await?;

        #[cfg(debug_assertions)]
        log::debug!("response in: {:#?}", now.elapsed());

        self.handle_response(response).await
    }
}

use crate::core::traits::HttpClient;
use crate::core::types::QResponse;
use async_trait::async_trait;

pub struct RealHttpClient;

#[async_trait]
impl HttpClient for RealHttpClient {
    async fn send_request(
        &self,
        method: &str,
        url: &str,
        headers: Vec<(String, String)>,
        body: Option<String>,
        query: Vec<(String, String)>,
    ) -> Result<QResponse, String> {
        // Print final URL with query params
        if let Ok(mut parsed_url) = url::Url::parse(url) {
            if !query.is_empty() {
                parsed_url.query_pairs_mut().extend_pairs(query.iter());
            }
            println!(
                "🚀 Sending Request: {} {}",
                method.to_uppercase(),
                parsed_url
            );
        } else {
            println!("🚀 Sending Request: {} {}", method.to_uppercase(), url);
        }

        let client = reqwest::Client::new();
        let mut builder = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            "HEAD" => client.head(url),
            _ => return Err(format!("Unsupported method: {}", method)),
        };

        for (name, value) in headers {
            builder = builder.header(name, value);
        }

        if !query.is_empty() {
            builder = builder.query(&query);
        }

        if let Some(b) = body {
            builder = builder.body(b);
        }

        let start = std::time::Instant::now();
        let response = builder.send().await.map_err(|e| e.to_string())?;
        let elapsed = start.elapsed().as_millis() as u64;

        let status = response.status().as_u16();
        let status_text = response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .to_string();

        let mut res_headers = Vec::new();
        for (name, value) in response.headers() {
            res_headers.push(crate::core::types::Header {
                name: name.to_string(),
                value: value.to_str().unwrap_or_default().to_string(),
                enabled: true,
                secret_key: None,
                r#type: "plain".to_string(),
            });
        }

        let body_content = response.text().await.map_err(|e| e.to_string())?;
        let size = body_content.len() as u64;

        Ok(QResponse {
            status,
            status_text,
            headers: res_headers,
            body: body_content,
            error: None,
            time_elapsed: elapsed,
            size,
        })
    }
}

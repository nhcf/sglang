use std::sync::Arc;

use axum::{extract::Request, http::HeaderMap, response::Response, body};
use reqwest::Client;
use tracing::{error, info};

use crate::{app_context::AppContext, core::worker::Worker};

#[derive(Clone)]
pub struct DiffusionProxy {
    client: Client,
    context: Arc<AppContext>,
}

impl DiffusionProxy {
    pub fn new(context: Arc<AppContext>) -> Self {
        Self {
            client: Client::new(),
            context,
        }
    }

    async fn select_worker(&self) -> Option<Arc<dyn Worker>> {
        let workers = self.context.worker_registry.get_all();
        
        if workers.is_empty() {
            error!("No workers available for diffusion proxy");
            return None;
        }

        // 随机选择一个工作器（不依赖健康检查）
        use std::time::SystemTime;
        let random_index = SystemTime::now()
            .elapsed()
            .unwrap_or_default()
            .subsec_nanos() as usize % workers.len();
        Some(workers[random_index].clone())
    }

    pub async fn proxy_request(&self, req: Request, headers: Option<&HeaderMap>) -> Response {
        match self.select_worker().await {
            Some(worker) => {
                let worker_url = worker.url().to_string();
                info!("Forwarding request to worker: {}", worker_url);
                
                // 构建转发请求
                let mut forward_req = self.client
                    .request(req.method().clone(), format!("{}{}", worker_url, req.uri()))
                    .version(req.version());
                
                // 传递请求头
                if let Some(headers) = headers {
                    for (key, value) in headers {
                        if !key.as_str().to_lowercase().starts_with("host") {
                            forward_req = forward_req.header(key, value);
                        }
                    }
                }
                
                // 传递请求体
                let body = req.into_body();
                let body = body::to_bytes(body, 10 * 1024 * 1024).await.unwrap_or_default();
                forward_req = forward_req.body(body);
                
                // 发送请求并处理响应
                match forward_req.send().await {
                    Ok(resp) => {
                        let status = resp.status();
                        let headers = resp.headers().clone();
                        let body = resp.bytes().await.unwrap_or_default();
                        
                        // 构建响应
                        let mut response = Response::new(body.into());
                        *response.status_mut() = status;
                        *response.headers_mut() = headers;
                        response
                    }
                    Err(e) => {
                        error!("Failed to forward request to worker: {}", e);
                        Response::builder()
                            .status(503)
                            .body("Service Unavailable".into())
                            .unwrap()
                    }
                }
            }
            None => {
                error!("No healthy workers available");
                Response::builder()
                    .status(503)
                    .body("Service Unavailable".into())
                    .unwrap()
            }
        }
    }
}

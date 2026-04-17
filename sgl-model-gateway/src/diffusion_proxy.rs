use std::sync::Arc;

use axum::{extract::Request, http::HeaderMap, response::Response, body};
use reqwest::Client;
use tracing::{error, info};

use crate::app_context::AppContext;

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

    fn select_worker_url(&self) -> Option<String> {
        // 从 worker_registry 中获取健康的 worker
        let healthy_workers = self.context.worker_registry.get_workers_filtered(
            None, // 不按模型过滤
            None, // 不按 worker 类型过滤
            None, // 不按连接模式过滤
            None, // 不按运行时类型过滤
            true, // 只获取健康的 worker
        );

        if healthy_workers.is_empty() {
            error!("No healthy workers available for diffusion proxy");
            return None;
        }

        // 随机选择一个健康的 worker
        use std::time::SystemTime;
        let random_index = SystemTime::now()
            .elapsed()
            .unwrap_or_default()
            .subsec_nanos() as usize % healthy_workers.len();
        Some(healthy_workers[random_index].url().to_string())
    }

    pub async fn proxy_request(&self, req: Request, headers: Option<&HeaderMap>) -> Response {
        match self.select_worker_url() {
            Some(worker_url) => {
                let method = req.method().clone();
                let uri = req.uri().clone();
                info!("Forwarding {} request to worker: {}{}", method, worker_url, uri);
                
                // 构建转发请求
                let mut forward_req = self.client
                    .request(method.clone(), format!("{}{}", worker_url, uri))
                    .version(req.version());
                
                // 传递请求头
                if let Some(headers) = headers {
                    for (key, value) in headers {
                        if !key.as_str().to_lowercase().starts_with("host") {
                            forward_req = forward_req.header(key, value);
                        }
                    }
                }
                
                // 根据请求方法处理请求体
                let method_str = method.as_str();
                let has_body = method_str == "POST" || method_str == "PUT" || method_str == "PATCH";
                
                let body = req.into_body();
                if has_body {
                    let body_bytes = body::to_bytes(body, 10 * 1024 * 1024).await.unwrap_or_default();
                    forward_req = forward_req.body(body_bytes);
                } else {
                    forward_req = forward_req.body(String::new());
                }
                
                // 发送请求并处理响应
                match forward_req.send().await {
                    Ok(resp) => {
                        let status = resp.status();
                        let headers = resp.headers().clone();
                        let body = resp.bytes().await.unwrap_or_default();
                        
                        info!("Worker response status: {}", status);
                        
                        // 构建响应
                        let mut response = Response::new(body.into());
                        *response.status_mut() = status;
                        *response.headers_mut() = headers;
                        response
                    }
                    Err(e) => {
                        error!("Failed to forward request to worker: error={}, url={}{}", e, worker_url, uri);
                        
                        // 根据错误类型返回不同的状态码
                        let status_code = if e.is_connect() || e.is_timeout() {
                            503 // Service Unavailable
                        } else if e.is_request() {
                            502 // Bad Gateway
                        } else {
                            500 // Internal Server Error
                        };
                        
                        let error_message = format!("Failed to forward request: {}", e);
                        Response::builder()
                            .status(status_code)
                            .body(error_message.into())
                            .unwrap()
                    }
                }
            }
            None => {
                error!("No healthy workers available");
                Response::builder()
                    .status(503)
                    .body("No healthy workers available".into())
                    .unwrap()
            }
        }
    }
}

mod common;

use axum::{extract::Request};

use sgl_model_gateway::{diffusion_proxy::DiffusionProxy};

// 通用测试函数，测试请求转发
async fn test_proxy_request(method: &str, uri: &str, body: Option<String>, content_type: Option<&str>) {
    let context = common::create_test_context(sgl_model_gateway::config::RouterConfig::default()).await;
    let proxy = DiffusionProxy::new(context);

    let mut request_builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", "Bearer sk-proj-1234567890");

    if let Some(ct) = content_type {
        request_builder = request_builder.header("Content-Type", ct);
    }

    let request = match body {
        Some(body) => request_builder.body(axum::body::Body::from(body)).unwrap(),
        None => request_builder.body(axum::body::Body::empty()).unwrap(),
    };

    let headers = request.headers().clone();
    let response = proxy.proxy_request(request, Some(&headers)).await;

    // 由于没有配置 worker URLs，应该返回 503 Service Unavailable
    assert_eq!(response.status(), 503);
}

#[tokio::test]
async fn test_diffusion_proxy_models() {
    // 测试模型信息接口
    test_proxy_request("GET", "/models", None, None).await;
}

#[tokio::test]
async fn test_diffusion_proxy_images_generations() {
    // 测试图像生成接口
    test_proxy_request(
        "POST",
        "/v1/images/generations",
        Some(r#"{"prompt": "A beautiful sunset", "size": "512x512"}"#.to_string()),
        Some("application/json"),
    ).await;
}

#[tokio::test]
async fn test_diffusion_proxy_images_edits() {
    // 测试图像编辑接口
    test_proxy_request(
        "POST",
        "/v1/images/edits",
        Some(r#"{"image": "@local_input_image.png", "prompt": "A beautiful sunset", "size": "512x512"}"#.to_string()),
        Some("multipart/form-data"),
    ).await;
}

#[tokio::test]
async fn test_diffusion_proxy_images_content() {
    // 测试图像下载接口
    test_proxy_request("GET", "/v1/images/test_image_id/content", None, None).await;
}

#[tokio::test]
async fn test_diffusion_proxy_videos() {
    // 测试视频生成接口
    test_proxy_request(
        "POST",
        "/v1/videos",
        Some(r#"{"prompt": "A beautiful sunset", "size": "1280x720"}"#.to_string()),
        Some("application/json"),
    ).await;
}

#[tokio::test]
async fn test_diffusion_proxy_videos_list() {
    // 测试视频列表接口
    test_proxy_request("GET", "/v1/videos", None, None).await;
}

#[tokio::test]
async fn test_diffusion_proxy_videos_content() {
    // 测试视频下载接口
    test_proxy_request("GET", "/v1/videos/test_video_id/content", None, None).await;
}

#[tokio::test]
async fn test_diffusion_proxy_set_lora() {
    // 测试 LoRA 设置接口
    test_proxy_request(
        "POST",
        "/v1/set_lora",
        Some(r#"{"lora_nickname": "test_lora", "lora_path": "/path/to/lora.safetensors", "target": "all", "strength": 0.8}"#.to_string()),
        Some("application/json"),
    ).await;
}

#[tokio::test]
async fn test_diffusion_proxy_merge_lora_weights() {
    // 测试 LoRA 合并接口
    test_proxy_request(
        "POST",
        "/v1/merge_lora_weights",
        Some(r#"{"target": "all", "strength": 0.8}"#.to_string()),
        Some("application/json"),
    ).await;
}

#[tokio::test]
async fn test_diffusion_proxy_unmerge_lora_weights() {
    // 测试 LoRA 解合并接口
    test_proxy_request(
        "POST",
        "/v1/unmerge_lora_weights",
        Some(r#"{}"#.to_string()),
        Some("application/json"),
    ).await;
}

#[tokio::test]
async fn test_diffusion_proxy_list_loras() {
    // 测试 LoRA 列表接口
    test_proxy_request("GET", "/v1/list_loras", None, None).await;
}

use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Testing diffusion proxy functionality...");
    
    // Create a reqwest client
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create client");
    
    // Test image generation endpoint
    println!("\nTesting image generation endpoint...");
    let image_request = serde_json::json!({
        "prompt": "A cat",
        "n": 1,
        "size": "1024x1024"
    });
    
    let image_response = client
        .post("http://127.0.0.1:30010/v1/images/generations")
        .json(&image_request)
        .send()
        .await;
    
    match image_response {
        Ok(resp) => {
            println!("Image generation response status: {}", resp.status());
            let body = resp.text().await.unwrap();
            println!("Response body: {}", body);
        }
        Err(e) => {
            println!("Error calling image generation endpoint: {}", e);
        }
    }
    
    // Test LoRA management endpoint
    println!("\nTesting LoRA management endpoint...");
    let lora_request = serde_json::json!({
        "name": "test_lora",
        "uri": "https://example.com/lora.safetensors"
    });
    
    let lora_response = client
        .post("http://127.0.0.1:30010/v1/diffusion/lora/adapters")
        .json(&lora_request)
        .send()
        .await;
    
    match lora_response {
        Ok(resp) => {
            println!("LoRA management response status: {}", resp.status());
            let body = resp.text().await.unwrap();
            println!("Response body: {}", body);
        }
        Err(e) => {
            println!("Error calling LoRA management endpoint: {}", e);
        }
    }
    
    println!("\nProxy test completed.");
}
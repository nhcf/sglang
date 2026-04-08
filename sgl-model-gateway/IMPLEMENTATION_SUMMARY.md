# 扩散模型透传功能实现总结

## 1. 实现的功能

### 1.1 核心功能
- **透传中间件**：实现了扩散模型相关接口的请求透传功能
- **工作器选择**：随机从健康的工作器中选择一个进行请求转发
- **请求转发**：将完整的请求（包括头信息和请求体）转发到选定的工作器
- **响应处理**：将工作器的响应完整返回给客户端
- **错误处理**：处理工作器不可用的情况，返回适当的错误信息

### 1.2 支持的接口
- **图像生成**：`POST /v1/images/generations`
- **图像编辑**：`POST /v1/images/edits`
- **图像下载**：`GET /v1/images/{image_id}/content`
- **视频生成**：`POST /v1/videos`
- **视频列表**：`GET /v1/videos`
- **视频下载**：`GET /v1/videos/{video_id}/content`
- **LoRA管理**：
  - `POST /v1/set_lora`
  - `POST /v1/merge_lora_weights`
  - `POST /v1/unmerge_lora_weights`
  - `GET /v1/list_loras`

## 2. 实现细节

### 2.1 代码结构
- **`src/diffusion_proxy.rs`**：实现了透传中间件的核心逻辑
- **`src/server.rs`**：添加了扩散模型相关接口的路由处理
- **`src/lib.rs`**：添加了 diffusion_proxy 模块的导入

### 2.2 工作流程
1. **请求接收**：服务器接收到扩散模型相关接口的请求
2. **工作器选择**：从健康的工作器中随机选择一个
3. **请求转发**：将请求转发到选定的工作器
4. **响应处理**：将工作器的响应返回给客户端
5. **错误处理**：如果工作器不可用，返回 503 错误

### 2.3 技术实现
- **HTTP客户端**：使用 reqwest 库实现 HTTP 请求转发
- **工作器选择**：基于现有的工作器注册和健康检查机制
- **请求处理**：保持请求的完整性，包括头信息和请求体
- **响应处理**：保持响应的完整性，包括状态码和响应体
- **错误处理**：实现适当的错误处理和日志记录

## 3. 使用方法

### 3.1 启动服务
1. **安装依赖**：确保系统中安装了 Rust 工具链
2. **构建项目**：`cargo build --release`
3. **启动服务**：
   ```bash
   ./target/release/smg launch --worker-urls http://127.0.0.1:30010
   ```
   其中 `http://127.0.0.1:30010` 是本地启动的 worker 实例地址

### 3.2 测试接口
- **图像生成**：
  ```bash
  curl -sS -X POST "http://localhost:30000/v1/images/generations" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer sk-proj-1234567890" \
    -d '{
          "prompt": "A calico cat playing a piano on stage",
          "size": "1024x1024",
          "n": 1,
          "response_format": "b64_json"
        }'
  ```

- **视频生成**：
  ```bash
  curl -sS -X POST "http://localhost:30000/v1/videos" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer sk-proj-1234567890" \
    -d '{
          "prompt": "A calico cat playing a piano on stage",
          "size": "1280x720"
        }'
  ```

- **LoRA设置**：
  ```bash
  curl -X POST http://localhost:30000/v1/set_lora \
    -H "Content-Type: application/json" \
    -d '{
          "lora_nickname": "lora_name",
          "lora_path": "/path/to/lora.safetensors",
          "target": "all",
          "strength": 0.8
        }'
  ```

## 4. 配置选项

### 4.1 启动参数
- **`--worker-urls`**：指定工作器的 URL 列表，例如 `http://127.0.0.1:30010`
- **`--port`**：指定网关服务的端口，默认 30000
- **`--policy`**：指定负载均衡策略，默认 cache_aware
- **`--api-key`**：指定 API 密钥，用于认证

### 4.2 环境要求
- **Rust 工具链**：需要安装 Rust 1.60+ 版本
- **Worker 实例**：需要在本地或远程启动扩散模型的 worker 实例，端口 30010

## 5. 故障排除

### 5.1 常见问题
- **工作器不可用**：检查 worker 实例是否正常运行，端口是否正确
- **认证失败**：检查 API 密钥是否正确，请求头是否包含 `Authorization: Bearer {api_key}`
- **请求失败**：检查请求参数是否正确，工作器是否支持该接口

### 5.2 日志查看
- 服务启动后，会在控制台输出日志信息
- 可以通过 `--log-level` 参数调整日志级别，例如 `--log-level debug`

## 6. 总结

本次实现了扩散模型相关接口的透传功能，将请求随机转发到可用的 worker 实例上，实现了请求的透明传递和处理。该功能的实现使得 SGLang Model Gateway 能够支持扩散模型的推理服务，为用户提供统一的 API 访问体验。

通过本次实现，SGLang Model Gateway 不仅支持语言模型的推理服务，还扩展了对扩散模型的支持，为用户提供更加全面的模型推理服务。
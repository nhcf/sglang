## 概述
SGLang Diffusion HTTP 服务实现了兼容 OpenAI 的 API，支持**图像/视频生成**、**LoRA 适配器管理**，同时可调整生成内容的输出质量与压缩级别。服务端基于 `sglang serve` 启动，支持多 GPU 部署与模型参数优化配置。

**前置要求**：使用 OpenAI Python SDK 时，需安装 Python 3.11+。

## 服务启动
### 启动命令
通过 `sglang serve` 命令启动服务，支持多参数配置，示例如下：

```bash
SERVER_ARGS=(
  --model-path ./nunchaku-sana  # 模型路径/模型ID
  --text-encoder-cpu-offload  # 文本编码器CPU卸载
  --pin-cpu-memory  # 固定CPU内存
  --num-gpus 4  # 使用GPU数量
  --ulysses-degree=2  # Ulysses并行度
  --ring-degree=2  # Ring并行度
  --port 30010  # 监听端口，默认30000
  --dry-run
)
sglang serve "${SERVER_ARGS[@]}"
```

### 核心启动参数
| 参数 | 说明 | 默认值 |
| --- | --- | --- |
| `--model-path` | 模型文件路径或 Hugging Face 模型 ID | 无（必填） |
| `--port` | HTTP 服务监听端口 | 30000 |
| `--num-gpus` | 启用的 GPU 数量 | 1 |


## 基础接口
### 获取模型信息
**接口地址**：`GET /models`  
**接口说明**：返回当前服务加载的模型信息，包括模型路径、任务类型、流水线配置、精度设置等。  
**请求示例**（curl）：

```bash
curl -sS -X GET "http://localhost:30010/models"
```

**响应示例**：

```json
{
  "model_path": "Wan-AI/Wan2.1-T2V-1.3B-Diffusers",
  "task_type": "T2V",
  "pipeline_name": "wan_pipeline",
  "pipeline_class": "WanPipeline",
  "num_gpus": 4,
  "dit_precision": "bf16",
  "vae_precision": "fp16"
}
```

## 图像生成接口
所有图像接口位于 `/v1/images` 命名空间，兼容 OpenAI Images API 规范，支持**图像生成**、**图像编辑**、**生成结果下载**。

### 1. 生成图像
**接口地址**：`POST /v1/images/generations`  
**请求方式**：POST  
**请求头**：

+ `Content-Type: application/json`
+ `Authorization: Bearer {api_key}`（示例：sk-proj-1234567890）  
**请求参数**：

| 参数 | 类型 | 说明 | 必填 | 默认值 |
| --- | --- | --- | --- | --- |
| `prompt` | string | 图像生成提示词 | 是 | 无 |
| `size` | string | 生成图像分辨率，如 1024x1024 | 是 | 无 |
| `n` | int | 生成图像数量 | 否 | 1 |
| `response_format` | string | 响应格式，可选 `b64_json`/`url` | 否 | `b64_json` |
| `output-quality` | string | 输出质量预设，可选 `maximum`/`high`/`medium`/`low`/`default` | 否 | `default` |
| `output-compression` | int | 压缩级别（0-100），优先级高于 `output-quality` | 否 | None |


**请求示例**（curl）：

```bash
curl -sS -X POST "http://localhost:30010/v1/images/generations" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-proj-1234567890" \
  -d '{
        "prompt": "A calico cat playing a piano on stage",
        "size": "1024x1024",
        "n": 1,
        "response_format": "b64_json"
      }'
```

**Python 示例**（b64_json 格式）：

```python
import base64
from openai import OpenAI

client = OpenAI(api_key="sk-proj-1234567890", base_url="http://localhost:30010/v1")
img = client.images.generate(
    prompt="A calico cat playing a piano on stage",
    size="1024x1024",
    n=1,
    response_format="b64_json",
)
# 解码并保存图像
image_bytes = base64.b64decode(img.data[0].b64_json)
with open("output.png", "wb") as f:
    f.write(image_bytes)
```

**注意**：若 `response_format=url` 且未配置云存储，返回相对下载地址：`/v1/images/<IMAGE_ID>/content`。

### 2. 编辑图像
**接口地址**：`POST /v1/images/edits`  
**请求方式**：POST  
**请求头**：`Authorization: Bearer {api_key}`  
**请求体**：multipart/form-data 格式，支持本地图像上传或网络图像 URL。  
**请求参数**：

| 参数 | 类型 | 说明 | 必填 |
| --- | --- | --- | --- |
| `image` | file | 本地输入图像（multipart 上传） | 二选一 |
| `url` | string | 网络图像 URL | 二选一 |
| `prompt` | string | 图像编辑提示词 | 是 |
| `size` | string | 编辑后图像分辨率 | 是 |
| `response_format` | string | 响应格式，`b64_json`/`url` | 否 |
| `output-quality` | string | 输出质量预设 | 否 |
| `output-compression` | int | 压缩级别（0-100） | 否 |


**请求示例**（curl，b64_json 响应）：

```bash
curl -sS -X POST "http://localhost:30010/v1/images/edits" \
  -H "Authorization: Bearer sk-proj-1234567890" \
  -F "image=@local_input_image.png" \
  -F "prompt=A calico cat playing a piano on stage" \
  -F "size=1024x1024" \
  -F "response_format=b64_json"
```

### 3. 下载生成/编辑的图像
**接口地址**：`GET /v1/images/{image_id}/content`  
**请求方式**：GET  
**请求头**：`Authorization: Bearer {api_key}`  
**路径参数**：`image_id` - 图像生成/编辑返回的唯一ID  
**请求示例**（curl）：

```bash
curl -sS -L "http://localhost:30010/v1/images/<IMAGE_ID>/content" \
  -H "Authorization: Bearer sk-proj-1234567890" \
  -o output.png
```

## 视频生成接口
所有视频接口位于 `/v1/videos` 命名空间，实现 OpenAI Videos API 子集，支持**视频生成**、**视频列表查询**、**视频下载**。

### 1. 生成视频
**接口地址**：`POST /v1/videos`  
**请求方式**：POST  
**请求头**：

+ `Content-Type: application/json`
+ `Authorization: Bearer {api_key}`  
**请求参数**：

| 参数 | 类型 | 说明 | 必填 |
| --- | --- | --- | --- |
| `prompt` | string | 视频生成提示词 | 是 |
| `size` | string | 视频分辨率，如 1280x720 | 是 |
| `output-quality` | string | 输出质量预设 | 否 |
| `output-compression` | int | 压缩级别（0-100） | 否 |


**请求示例**（curl）：

```bash
curl -sS -X POST "http://localhost:30010/v1/videos" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-proj-1234567890" \
  -d '{
        "prompt": "A calico cat playing a piano on stage",
        "size": "1280x720"
      }'
```

**Python 示例**：

```python
from openai import OpenAI
client = OpenAI(api_key="sk-proj-1234567890", base_url="http://localhost:30010/v1")
video = client.videos.create(
    prompt="A calico cat playing a piano on stage",
    size="1280x720"
)
print(f"Video ID: {video.id}, Status: {video.status}")
```

**响应说明**：返回视频 ID 与生成状态（如 `pending`/`completed`/`failed`），视频生成为异步过程，需轮询状态确认完成。

### 2. 查询视频列表
**接口地址**：`GET /v1/videos`  
**请求方式**：GET  
**请求头**：`Authorization: Bearer {api_key}`  
**接口说明**：返回所有生成任务的视频 ID 与当前状态，用于轮询视频生成进度。  
**Python 示例**：

```python
videos = client.videos.list()
for item in videos.data:
    print(item.id, item.status)
```

### 3. 下载生成的视频
**接口地址**：`GET /v1/videos/{video_id}/content`  
**请求方式**：GET  
**请求头**：`Authorization: Bearer {api_key}`  
**路径参数**：`video_id` - 视频生成返回的唯一ID  
**前置条件**：视频状态为 `completed` 时方可下载  
**Python 示例**（轮询+下载）：

```python
import time
from openai import OpenAI

client = OpenAI(api_key="sk-proj-1234567890", base_url="http://localhost:30010/v1")
video_id = "xxx"  # 生成视频返回的ID

# 轮询直到生成完成
while True:
    page = client.videos.list()
    item = next((v for v in page.data if v.id == video_id), None)
    if item and item.status == "completed":
        break
    time.sleep(5)

# 下载视频
resp = client.videos.download_content(video_id=video_id)
with open("output.mp4", "wb") as f:
    f.write(resp.read())
```

**curl 示例**：

```bash
curl -sS -L "http://localhost:30010/v1/videos/<VIDEO_ID>/content" \
  -H "Authorization: Bearer sk-proj-1234567890" \
  -o output.mp4
```

## LoRA 管理接口
支持 LoRA 适配器的**动态加载**、**合并**、**解合并**、**列表查询**，核心规则：

1. 同一时间仅能有一个 LoRA 处于合并（激活）状态；
2. 切换 LoRA 需先解合并当前激活的 LoRA，再加载新 LoRA；
3. 已加载的 LoRA 权重会缓存至内存，重新激活无额外开销。

### 1. 设置/加载 LoRA 适配器
**接口地址**：`POST /v1/set_lora`  
**请求方式**：POST  
**请求头**：`Content-Type: application/json`  
**接口说明**：加载单个/多个 LoRA 适配器并将权重合并到基础模型，首次加载需指定 `lora_path`，缓存后可仅通过 `lora_nickname` 激活。  
**请求参数**：

| 参数 | 类型 | 说明 | 必填 | 默认值 |
| --- | --- | --- | --- | --- |
| `lora_nickname` | string/list | LoRA 唯一标识，多 LoRA 时为列表 | 是 | 无 |
| `lora_path` | string/list/None | LoRA 文件路径（.safetensors）/Hugging Face ID，多 LoRA 时为列表；缓存后可省略 | 首次加载是 | None |
| `target` | string/list | 应用 LoRA 的Transformer，可选 `all`/`transformer`/`transformer_2`/`critic`；多 LoRA 时为列表 | 否 | `all` |
| `strength` | float/list | LoRA 融合强度，<1 减弱效果，>1 增强效果；多 LoRA 时为列表 | 否 | 1.0 |


**单 LoRA 请求示例**（curl）：

```bash
curl -X POST http://localhost:30010/v1/set_lora \
  -H "Content-Type: application/json" \
  -d '{
        "lora_nickname": "lora_name",
        "lora_path": "/path/to/lora.safetensors",
        "target": "all",
        "strength": 0.8
      }'
```

**多 LoRA 请求示例**（curl）：

```bash
curl -X POST http://localhost:30010/v1/set_lora \
  -H "Content-Type: application/json" \
  -d '{
        "lora_nickname": ["lora_1", "lora_2"],
        "lora_path": ["/path/to/lora1.safetensors", "/path/to/lora2.safetensors"],
        "target": ["transformer", "transformer_2"],
        "strength": [0.8, 1.0]
      }'
```

**多 LoRA 注意事项**：

+ 列表参数（`lora_nickname`/`lora_path`/`target`/`strength`）长度必须一致；
+ 若 `target`/`strength` 为单个值，将应用于所有 LoRA；
+ 同一 `target` 上的多 LoRA 按列表顺序融合。

### 2. 手动合并 LoRA 权重
**接口地址**：`POST /v1/merge_lora_weights`  
**请求方式**：POST  
**请求头**：`Content-Type: application/json`  
**接口说明**：将当前已加载的 LoRA 权重手动合并到基础模型；`set_lora` 会自动执行合并，此接口仅用于手动解合并后重新融合。  
**请求参数**：

| 参数 | 类型 | 说明 | 默认值 |
| --- | --- | --- | --- |
| `target` | string | 合并的Transformer，可选 `all`/`transformer`/`transformer_2`/`critic` | `all` |
| `strength` | float | 融合强度 | 1.0 |


**请求示例**（curl）：

```bash
curl -X POST http://localhost:30010/v1/merge_lora_weights \
  -H "Content-Type: application/json" \
  -d '{"strength": 0.8}'
```

### 3. 解合并 LoRA 权重
**接口地址**：`POST /v1/unmerge_lora_weights`  
**请求方式**：POST  
**请求头**：`Content-Type: application/json`  
**接口说明**：将当前激活的 LoRA 权重从基础模型中解合并，恢复模型原始状态；**切换 LoRA 前必须调用此接口**。  
**请求示例**（curl）：

```bash
curl -X POST http://localhost:30010/v1/unmerge_lora_weights \
  -H "Content-Type: application/json"
```

### 4. 查询 LoRA 适配器列表
**接口地址**：`GET /v1/list_loras`  
**请求方式**：GET  
**接口说明**：返回已加载的 LoRA 适配器信息及各模块的当前激活状态。  
**请求示例**（curl）：

```bash
curl -sS -X GET "http://localhost:30010/v1/list_loras"
```

**响应示例**：

```json
{
  "loaded_adapters": [
    { "nickname": "lora_a", "path": "/weights/lora_a.safetensors" },
    { "nickname": "lora_b", "path": "/weights/lora_b.safetensors" }
  ],
  "active": {
    "transformer": [
      {
        "nickname": "lora2",
        "path": "tarn59/pixel_art_style_lora_z_image_turbo",
        "merged": true,
        "strength": 1.0
      }
    ]
  }
}
```

### LoRA 切换示例（curl）
```bash
# 1. 加载并激活 LoRA A
curl -X POST http://localhost:30010/v1/set_lora -d '{"lora_nickname": "lora_a", "lora_path": "path/to/A"}'
# 2. 使用 LoRA A 生成内容...
# 3. 解合并 LoRA A
curl -X POST http://localhost:30010/v1/unmerge_lora_weights
# 4. 加载并激活 LoRA B
curl -X POST http://localhost:30010/v1/set_lora -d '{"lora_nickname": "lora_b", "lora_path": "path/to/B"}'
# 5. 使用 LoRA B 生成内容...
```

## 输出质量与压缩配置
图像/视频生成接口均支持通过以下参数调整输出质量，**优先级**：`output-compression` > `output-quality`。

### 核心参数
| 参数 | 类型 | 可选值/范围 | 说明 |
| --- | --- | --- | --- |
| `output-quality` | string | `maximum`/`high`/`medium`/`low`/`default` | 质量预设，自动配置压缩级别：   maximum(100)、high(90)、medium(55)、low(35)、default（图像75/视频50） |
| `output-compression` | int | 0-100 | 手动压缩级别，0=最低质量（最小文件），100=最高质量（最大文件） |


### 注意事项
1. 质量配置仅对 **JPEG 图像** 和 **视频** 生效，PNG 为无损压缩，忽略该配置；
2. 低压缩级别（或 `low` 预设）会生成更小的文件，但可能出现视觉失真。

## 通用请求头
所有接口均需携带授权头：

```plain
Authorization: Bearer {api_key}
```

示例：`Authorization: Bearer sk-proj-1234567890`（api_key 可自定义，无严格校验规则）。


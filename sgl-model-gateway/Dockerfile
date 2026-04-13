# 第一阶段：构建阶段
FROM rustlang/rust:nightly-slim AS builder

# 安装必要的构建依赖
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    git \
    ca-certificates \
    libssl-dev \
    protobuf-compiler \
    libprotobuf-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制项目文件
COPY . .

# 构建项目（使用 release 模式）
RUN cargo build --release

# 第二阶段：运行阶段
FROM rustlang/rust:nightly-slim

# 安装必要的依赖
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 创建用户和组以非 root 身份运行
RUN groupadd -r sglang && useradd -r -g sglang sglang

# 设置工作目录
WORKDIR /app

# 从构建阶段复制构建产物
COPY --from=builder /app/target/release/sgl-model-gateway /app/
COPY --from=builder /app/target/release/smg /app/
COPY --from=builder /app/target/release/amg /app/

# 设置权限
RUN chown -R sglang:sglang /app

# 切换到非 root 用户
USER sglang

# 暴露端口（默认端口 30000）
EXPOSE 30000

# 设置默认命令
CMD ["./smg", "launch", "--host", "0.0.0.0", "--port", "30000"]

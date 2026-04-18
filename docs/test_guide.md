## Features 说明

### ylong\_http\_client features

| Feature         | 说明                                 |
| --------------- | ---------------------------------- |
| `sync`          | 同步接口                               |
| `async`         | 异步接口                               |
| `http1_1`       | HTTP/1.1                           |
| `http2`         | HTTP/2                             |
| `http3`         | HTTP/3                             |
| `tokio_base`    | 使用 tokio runtime（与 ylong\_base 互斥） |
| `ylong_base`    | 使用 ylong runtime（与 tokio\_base 互斥） |
| `__tls`         | TLS 支持（需要配合具体 TLS 实现）              |
| `__c_openssl`   | C 语言 OpenSSL 实现                    |
| `c_openssl_3_0` | OpenSSL 3.0（需要配合 \_\_c\_openssl）   |

### ylong\_http features

| Feature      | 说明                                 |
| ------------ | ---------------------------------- |
| `http1_1`    | HTTP/1.1                           |
| `http2`      | HTTP/2                             |
| `http3`      | HTTP/3                             |
| `huffman`    | Huffman编码                          |
| `tokio_base` | 使用 tokio（与 ylong\_base 互斥）         |
| `ylong_base` | 使用 ylong runtime（与 tokio\_base 互斥） |

> **注意**: `tokio_base` 和 `ylong_base` 互斥，不能同时启用。

***

## 示例说明

### ⚠️ 不可用示例（sync 代码未完成）

以下示例因 sync 模块代码未完成暂时无法运行：

- **sync\_redirect\_http** - sync 代码未完成
- **sync\_proxy\_http** - sync 代码未完成
- **sync\_http** - sync 代码未完成
- **sync\_https\_outside** - sync 代码未完成

### ✅ 可用示例 (8个)

#### 普通示例 (5个)

1. **async\_http.rs** - 异步HTTP
   - 依赖: `async`, `http1_1`, `ylong_base`
2. **async\_http\_dns.rs** - 异步HTTP + DNS
   - 依赖: `async`, `http1_1`, `ylong_base`
3. **async\_http\_multi.rs** - 异步HTTP多请求
   - 依赖: `async`, `http1_1`, `ylong_base`
4. **async\_proxy\_http.rs** - 异步HTTP代理
   - 依赖: `async`, `http1_1`, `tokio_base`
   - 需要本地 HTTP 服务器（127.0.0.1:3000）
5. **async\_redirect\_http.rs** - 异步HTTP重定向
   - 依赖: `async`, `http1_1`, `tokio_base`
   - 需要本地 HTTP 服务器（127.0.0.1:3000）

#### TLS 示例 (3个)

1. **async\_https\_outside.rs** - 异步HTTPS（外部）
   - 依赖: `async`, `http1_1`, `__tls`, `__c_openssl`, `c_openssl_3_0`, `tokio_base`
2. **async\_http\_doh.rs** - 异步HTTP + DoH (DNS over HTTPS)
   - 依赖: `async`, `http1_1`, `ylong_base`, `__c_openssl`, `c_openssl_3_0`
3. **async\_certs\_adapter.rs** - 异步证书适配器
   - 依赖: `async`, `http1_1`, `ylong_base`, `__c_openssl`, `c_openssl_3_0`

#### 运行示例

#### 前置准备

1. **安装 miniserve**（用于本地 HTTP 服务器）
   ```bash
   cargo install miniserve
   ```

2. **启动本地 HTTP 服务器**
   ```bash
   miniserve --port 3000 .
   ```

#### 运行命令

```bash
# 使用 test 脚本运行所有示例（自动配置 OpenSSL）
cd ylong_http_client && ./test all

# 运行单个示例
cd ylong_http_client && ./test async_http
```

***

## 单元测试

### 运行库测试

```bash
cd ylong_http_client

# 无 TLS
cargo test --lib --features "async,http1_1,ylong_base,libc"

# 有 TLS
cargo test --lib --features "async,http1_1,tls,ylong_base,libc"
```

### 测试结果

- **库测试**: 154 个测试全部通过
- **依赖本地服务器的示例**（如 `async_proxy_http`、`async_redirect_http`）需要本地 HTTP 服务器在 `127.0.0.1:3000` 运行，否则会报 "Connection refused"，这是正常行为

***

## E2E 测试

### HTTPS 代理 E2E 测试

使用 mitmproxy 容器模拟 HTTPS 代理服务器，验证以下功能：
- HTTP CONNECT 隧道建立
- HTTPS 代理（无 TLS / TLS 加密）
- mTLS（双向 TLS 认证）
- TLS 版本和加密套件配置

#### 前置要求

1. **Docker** - 需要 Docker 运行环境
2. **mitmproxy 镜像** - 测试会自动拉取
   ```
   swr.cn-north-4.myhuaweicloud.com/ddn-k8s/docker.io/mitmproxy/mitmproxy:11.0
   ```
3. **OpenSSL 环境配置**
   ```bash
   export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
   export OPENSSL_INCLUDE_DIR=/usr/include
   export RUSTFLAGS="-L $OPENSSL_LIB_DIR -l ssl -l crypto"
   ```

#### 运行流程

1. **验证 Docker 状态**
   ```bash
   docker --version
   systemctl status docker  # 检查 Docker 服务是否运行
   ```

2. **拉取 mitmproxy 镜像**（可选，测试会自动拉取）
   ```bash
   docker pull swr.cn-north-4.myhuaweicloud.com/ddn-k8s/docker.io/mitmproxy/mitmproxy:11.0
   ```

3. **运行 E2E 测试**
   ```bash
   cd ylong_http_client
   cargo test --test sdv_async_https_proxy_e2e --features "async,http1_1,tls,ylong_base,libc" -- --nocapture
   ```

#### 预期结果

```
✅ sdv_async_https_proxy               - 通过
✅ sdv_async_https_proxy_no_tls        - 通过
✅ sdv_async_https_proxy_tls           - 通过
✅ sdv_async_https_proxy_mitmproxy_full_config     - 通过
✅ sdv_async_https_proxy_mitmproxy_tls_version     - 通过
```

#### 故障排查

1. **"Connection refused" 错误**
   - Docker 容器可能未正常启动
   - 手动验证：`docker run --rm --network=host --entrypoint mitmdump mitmproxy:11.0 --listen-port 8888`

2. **Docker 网络问题**
   - 确保 Docker daemon 正在运行
   - 检查 `--network=host` 是否被支持

3. **镜像拉取慢**
   - 测试前手动拉取：`docker pull swr.cn-north-4.myhuaweicloud.com/ddn-k8s/docker.io/mitmproxy/mitmproxy:11.0`

4. **OpenSSL 链接错误**
   - 确保正确设置了 OpenSSL 环境变量
   - 检查 OpenSSL 库是否安装：`apt install libssl-dev`（Ubuntu/Debian）

***

## 编译指南

### 推荐编译命令

```bash
# 基础编译（无 TLS）
cargo build --features "async,http1_1,ylong_base,libc"

# TLS 编译（完整特性）
cargo build --features "async,http1_1,tls,ylong_base,libc"
```

### 编译问题排查

#### 问题 1: 缺失 `tunnel_over_proxy_tls` 函数

```
error[E0425]: cannot find function `tunnel_over_proxy_tls` in this scope
```

**解决方案**: 确保 TLS 模块中保留了 `tunnel_over_proxy_tls` 函数（用于 HTTPS 代理 + TLS 场景）

#### 问题 2: Future 不满足 `Sync` 约束

```
future created by async block is not `Sync`
```

**解决方案**: 为 `Tunnel` trait 的 `establish` 方法添加 `Sync` bound：
```rust
fn establish(...) -> Pin<Box<dyn Future<Output = Result<Self::Stream, TunnelError>> + Send + Sync + '_>>;
```

#### 问题 3: 关联类型 `Stream` 找不到

```
error[E0220]: associated type `Stream` not found for `C`
```

**解决方案**: 在使用 `C::Stream` 的文件中添加 `Tunnel` trait 导入：
```rust
#[cfg(feature = "__tls")]
use crate::proxy::tunnel::Tunnel;
```

#### 问题 4: TLS 链接失败

```
undefined symbol: SSL_get_rbio, SSL_read, SSL_write...
```

**解决方案**: 正确配置 OpenSSL 环境变量：
```bash
export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include
export RUSTFLAGS="-L $OPENSSL_LIB_DIR -l ssl -l crypto"
```

***

## 补充说明

### TLS 环境配置

运行 TLS 相关示例需要设置以下环境变量：

| 变量 | 说明 | Ubuntu/Debian 示例 |
|------|------|-------------------|
| `OPENSSL_LIB_DIR` | OpenSSL 库目录 | `/usr/lib/x86_64-linux-gnu` |
| `OPENSSL_INCLUDE_DIR` | OpenSSL 头文件目录 | `/usr/include` |
| `RUSTFLAGS` | 链接参数 | `-L $OPENSSL_LIB_DIR -l ssl -l crypto` |

> **注意**: `ylong_http_client/test` 脚本已自动配置这些环境变量，直接运行 `./test all` 即可。

### 常用命令速查

```bash
# 编译检查
cargo build --features "async,http1_1,tls,ylong_base,libc"

# 运行所有示例
cd ylong_http_client && ./test all

# 运行库测试
cargo test --lib --features "async,http1_1,tls,ylong_base,libc"

# 运行 HTTPS 代理 E2E 测试
cargo test --test sdv_async_https_proxy_e2e --features "async,http1_1,tls,ylong_base,libc"
```
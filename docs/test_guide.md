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

## ylong\_http\_client/examples

### ⚠️ 不可用示例（sync 代码未完成）

以下示例因 sync 模块代码未完成暂时无法运行：

- **sync\_redirect\_http** - sync 代码未完成
- **sync\_proxy\_http** - sync 代码未完成
- **sync\_http** - sync 代码未完成
- **sync\_https\_outside** - sync 代码未完成

### ✅ 可用示例 (8个)

#### 普通示例 (5个)

1. **async\_redirect\_http.rs** - 异步HTTP重定向
   - 依赖: `async`, `http1_1`, `tokio_base`
   - 需要本地 HTTP 服务器（127.0.0.1:3000）
2. **async\_proxy\_http.rs** - 异步HTTP代理
   - 依赖: `async`, `http1_1`, `tokio_base`
   - 需要本地 HTTP 服务器（127.0.0.1:3000）
3. **async\_http\_multi.rs** - 异步HTTP多请求
   - 依赖: `async`, `http1_1`, `ylong_base`
4. **async\_http\_dns.rs** - 异步HTTP + DNS
   - 依赖: `async`, `http1_1`, `ylong_base`
5. **async\_http.rs** - 异步HTTP
   - 依赖: `async`, `http1_1`, `ylong_base`

#### TLS 示例 (3个)

1. **async\_http\_doh.rs** - 异步HTTP + DoH (DNS over HTTPS)
   - 依赖: `async`, `http1_1`, `ylong_base`, `__c_openssl`, `c_openssl_3_0`
   - 需要配置 TLS 环境变量（见下方说明）
2. **async\_certs\_adapter.rs** - 异步证书适配器
   - 依赖: `async`, `http1_1`, `ylong_base`, `__c_openssl`, `c_openssl_3_0`
   - 需要配置 TLS 环境变量（见下方说明）
3. **async\_https\_outside.rs** - 异步HTTPS（外部）
   - 依赖: `async`, `http1_1`, `__tls`, `__c_openssl`, `c_openssl_3_0`, `tokio_base`
   - 需要配置 TLS 环境变量（见下方说明）

### 使用 tokio\_base 的 examples

- async\_redirect\_http, async\_proxy\_http, async\_https\_outside

### 使用 ylong\_base 的 examples

- async\_http, async\_http\_dns, async\_http\_multi, async\_http\_doh, async\_certs\_adapter

***

## Test 脚本说明

### ylong\_http\_client/test

运行所有可用示例：

```bash
cd ylong_http_client && ./test all
```

运行单个示例：

```bash
cd ylong_http_client && ./test async_http
```

#### 测试结果说明

- **编译成功，运行失败**：部分示例（如 async\_proxy\_http、async\_redirect\_http）需要本地 HTTP 服务器在 127.0.0.1:3000 运行，否则会报 "Connection refused" 错误。这是正常行为，不是代码问题。
- **TLS 示例**：需要正确配置 TLS 环境变量才能运行（见下方说明）。

#### 配置文件说明

脚本会自动添加 `libc` feature，无需手动指定。

### ylong\_http/test

运行所有示例：

```bash
cd ylong_http && ./test all
```

运行单个示例：

```bash
cd ylong_http && ./test mimebody_multi
cd ylong_http && ./test mimebody_multi_then_async_data
```

***

## cargo test 说明

### 运行单元测试

```bash
cd ylong_http_client
cargo test --features=async,http1_1,ylong_base
```

### 编译问题排查

#### 问题: HTTPS 代理场景编译失败

在启用 TLS 特性 (`__tls`) 时编译可能遇到以下错误：

1. **缺失 `tunnel_over_proxy_tls` 函数**
   ```
   error[E0425]: cannot find function `tunnel_over_proxy_tls` in this scope
   ```
   **解决方案**: 确保 TLS 模块中保留了 `tunnel_over_proxy_tls` 函数（用于 HTTPS 代理 + TLS 场景）

2. **Future 不满足 `Sync` 约束**
   ```
   future created by async block is not `Sync`
   ```
   **解决方案**: 为 `Tunnel` trait 的 `establish` 方法添加 `Sync` bound：
   ```rust
   fn establish(...) -> Pin<Box<dyn Future<Output = Result<Self::Stream, TunnelError>> + Send + Sync + '_>>;
   ```

3. **关联类型 `Stream` 找不到**
   ```
   error[E0220]: associated type `Stream` not found for `C`
   ```
   **解决方案**: 在使用 `C::Stream` 的文件中添加 `Tunnel` trait 导入：
   ```rust
   #[cfg(feature = "__tls")]
   use crate::proxy::tunnel::Tunnel;
   ```

#### 推荐编译命令

```bash
# 基础编译（无 TLS）
cargo build --features "async,http1_1,ylong_base,libc"

# TLS 编译（完整特性）
cargo build --features "async,http1_1,tls,ylong_base,libc"
```

##

***

## 补充说明

### TLS 运行要求

运行 TLS 相关示例需要设置以下环境变量：

- `OPENSSL_LIB_DIR` - OpenSSL 库目录
- `OPENSSL_INCLUDE_DIR` - OpenSSL 头文件目录
- `RUSTFLAGS` - 链接参数

**示例（Ubuntu/Debian）：**
```bash
export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include
export RUSTFLAGS="-L $OPENSSL_LIB_DIR -l ssl -l crypto"
```

> **注意**: `ylong_http_client/test` 脚本已自动配置这些环境变量，直接运行 `./test all` 即可。

### 测试结果说明

- **依赖本地服务器的示例**（如 `async_proxy_http`、`async_redirect_http`）需要本地 HTTP 服务器在 `127.0.0.1:3000` 运行，否则会报 "Connection refused"，这是正常行为
- **TLS 示例**（如 `async_https_outside`、`async_certs_adapter`）需要正确配置 OpenSSL 环境变量
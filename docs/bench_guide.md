# 基准测试指南

## 环境要求

运行 TLS 相关基准测试需要设置以下环境变量：

```bash
export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include
export RUSTFLAGS="-L $OPENSSL_LIB_DIR -l ssl -l crypto"
```

或者一行命令：

```bash
OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include RUSTFLAGS="-L /usr/lib/x86_64-linux-gnu -l ssl -l crypto" cargo run --example <bench_name> ...
```

## 基准测试用例

| 示例 | 说明 | 参数 |
|------|------|------|
| bench_all | 集成测试 | `[runs] [iterations]` |
| bench_proxy_connect | 连接延迟 | `[runs] [iterations]` |
| bench_proxy_timing | 分阶段延迟 | `[runs] [iterations]` |
| bench_proxy_throughput | 吞吐量 | `[runs] [concurrent] [duration]` |
| bench_proxy_reuse | 连接复用 | `[runs] [clients] [requests_per_client]` |

## 运行示例

```bash
# 运行集成测试（默认 10 次）
cd ylong_http_client
OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include RUSTFLAGS="-L /usr/lib/x86_64-linux-gnu -l ssl -l crypto" \
  cargo run --example bench_all --features "async,http1_1,ylong_base,tls,__c_openssl,c_openssl_3_0,libc"

# 运行指定次数
OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu OPENSSL_INCLUDE_DIR=/usr/include RUSTFLAGS="-L /usr/lib/x86_64-linux-gnu -l ssl -l crypto" \
  cargo run --example bench_all --features "..." -- 5 20
```

## 默认值

| 测试 | runs | iterations |
|------|------|----------|
| bench_all | 10 | 20 |
| bench_proxy_connect | 10 | 100 |
| bench_proxy_timing | 10 | 50 |
| bench_proxy_throughput | 10 | 10 (concurrent), 5s |
| bench_proxy_reuse | 10 | 5 (clients), 10 (requests) |
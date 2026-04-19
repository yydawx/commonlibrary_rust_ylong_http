use std::time::Instant;
use ylong_http_client::async_impl::{Body, Client, Request};
use ylong_http_client::HttpClientError;

const TARGET_URL: &str = "https://example.com";
const DEFAULT_RUNS: usize = 10;
const DEFAULT_CONCURRENT: usize = 10;
const DEFAULT_DURATION_SECS: u64 = 5;

fn main() -> Result<(), HttpClientError> {
    let args: Vec<String> = std::env::args().collect();
    let runs = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_RUNS);
    let concurrent = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_CONCURRENT);
    let duration = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_DURATION_SECS);

    println!("=== HTTPS Throughput Benchmark ===\n");
    println!("Runs: {}, Concurrent: {}, Duration: {}s\n", runs, concurrent, duration);

    let mut all_qps = Vec::new();

    for run in 0..runs {
        println!("Run {}/{}...", run + 1, runs);

        let mut handles = Vec::new();
        let start = Instant::now();

        for _ in 0..concurrent {
            let handle = ylong_runtime::spawn(make_requests(duration));
            handles.push(handle);
        }

        let mut total_requests = 0usize;
        for handle in handles {
            if let Ok(count) = ylong_runtime::block_on(handle) {
                total_requests += count;
            }
        }

        let elapsed = start.elapsed();
        let qps = total_requests as f64 / elapsed.as_secs_f64();
        all_qps.push(qps);

        println!("  QPS: {:.2}", qps);
    }

    println!("\n=== Summary ===\n");

    if !all_qps.is_empty() {
        let avg = all_qps.iter().sum::<f64>() / all_qps.len() as f64;
        println!("QPS avg: {:.2}", avg);
    }

    Ok(())
}

async fn make_requests(duration_secs: u64) -> usize {
    let client = match Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(_) => return 0,
    };

    let mut count = 0;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(duration_secs);

    while std::time::Instant::now() < deadline {
        let request = match Request::builder()
            .url(TARGET_URL)
            .body(Body::empty())
        {
            Ok(r) => r,
            Err(_) => continue,
        };

        if client.request(request).await.is_ok() {
            count += 1;
        }
    }

    count
}
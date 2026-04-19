use std::time::Instant;
use ylong_http_client::async_impl::{Body, Client, Request};
use ylong_http_client::HttpClientError;

const TARGET_URL: &str = "https://example.com";
const DEFAULT_RUNS: usize = 10;
const DEFAULT_CLIENT_COUNT: usize = 5;
const DEFAULT_REQUESTS_PER_CLIENT: usize = 10;

fn main() -> Result<(), HttpClientError> {
    let args: Vec<String> = std::env::args().collect();
    let runs = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_RUNS);
    let client_count = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_CLIENT_COUNT);
    let requests_per_client = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_REQUESTS_PER_CLIENT);

    println!("=== HTTPS Connection Reuse Benchmark ===\n");
    println!("Runs: {}, Clients: {}, Requests per client: {}\n", runs, client_count, requests_per_client);

    let mut all_new_avg = Vec::new();
    let mut all_reuse_avg = Vec::new();

    for run in 0..runs {
        println!("Run {}/{}...", run + 1, runs);

        let mut new_times = Vec::new();
        let mut reuse_times = Vec::new();

        for client_id in 0..client_count {
            let result = ylong_runtime::block_on(async {
                let client = Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()?;

                let mut first_time = None;
                let mut other_times = Vec::new();

                for i in 0..requests_per_client {
                    let req_start = Instant::now();

                    let request = Request::builder()
                        .url(TARGET_URL)
                        .body(Body::empty())?;

                    let _ = client.request(request).await;

                    let elapsed = req_start.elapsed();

                    if i == 0 {
                        first_time = Some(elapsed);
                    } else {
                        other_times.push(elapsed);
                    }
                }

                Ok::<_, HttpClientError>((first_time, other_times))
            });

            match result {
                Ok((first, mut others)) => {
                    if let Some(f) = first {
                        new_times.push(f);
                    }
                    reuse_times.append(&mut others);
                }
                Err(e) => {
                    println!("  Client {} error: {:?}", client_id, e);
                }
            }
        }

        if !new_times.is_empty() {
            let avg_new = new_times.iter().sum::<std::time::Duration>() / new_times.len() as u32;
            let avg_reuse = if !reuse_times.is_empty() {
                reuse_times.iter().sum::<std::time::Duration>() / reuse_times.len() as u32
            } else {
                std::time::Duration::ZERO
            };

            all_new_avg.push(avg_new);
            all_reuse_avg.push(avg_reuse);

            println!("  New: {:?}, Reuse: {:?}", avg_new, avg_reuse);
        }
    }

    println!("\n=== Summary ===\n");

    if !all_new_avg.is_empty() && !all_reuse_avg.is_empty() {
        let avg_new = all_new_avg.iter().sum::<std::time::Duration>() / all_new_avg.len() as u32;
        let avg_reuse = all_reuse_avg.iter().sum::<std::time::Duration>() / all_reuse_avg.len() as u32;
        let improvement = (avg_new.as_millis() as f64 - avg_reuse.as_millis() as f64) / avg_new.as_millis() as f64 * 100.0;

        println!("New avg: {:?}", avg_new);
        println!("Reuse avg: {:?}", avg_reuse);
        println!("Improvement: {:.1}%", improvement);
    }

    Ok(())
}
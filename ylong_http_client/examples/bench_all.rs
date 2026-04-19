use std::time::Instant;
use ylong_http_client::async_impl::{Body, Client, Request};
use ylong_http_client::HttpClientError;

const TARGET_URL: &str = "https://example.com";
const DEFAULT_RUNS: usize = 10;
const DEFAULT_ITERATIONS: usize = 20;

fn main() -> Result<(), HttpClientError> {
    let args: Vec<String> = std::env::args().collect();
    let runs = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_RUNS);
    let iterations = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_ITERATIONS);

    println!("=== HTTPS Connect Benchmark ===\n");
    println!("Runs: {}, Iterations per run: {}\n", runs, iterations);

    let mut all_times = Vec::new();

    for run in 0..runs {
        println!("Run {}/{}...", run + 1, runs);

        let mut run_times = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();

            let result = ylong_runtime::block_on(async {
                let client = Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()?;

                let request = Request::builder()
                    .url(TARGET_URL)
                    .body(Body::empty())?;

                client.request(request).await
            });

            let elapsed = start.elapsed();

            if result.is_ok() {
                run_times.push(elapsed);
            }
        }

        if !run_times.is_empty() {
            let avg = run_times.iter().sum::<std::time::Duration>() / run_times.len() as u32;
            all_times.push(avg);
            println!("  Avg: {:?}", avg);
        }
    }

    println!("\n=== Summary ===\n");

    if !all_times.is_empty() {
        let overall_avg = all_times.iter().sum::<std::time::Duration>() / all_times.len() as u32;
        let mut sorted = all_times.clone();
        sorted.sort();
        let min = sorted.first().unwrap();
        let max = sorted.last().unwrap();

        println!("Overall Average: {:?}", overall_avg);
        println!("Min: {:?}", min);
        println!("Max: {:?}", max);
    }

    Ok(())
}
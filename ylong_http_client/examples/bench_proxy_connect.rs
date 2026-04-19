use std::time::Instant;
use ylong_http_client::async_impl::{Body, Client, Request};
use ylong_http_client::HttpClientError;

const TARGET_URL: &str = "https://example.com";
const DEFAULT_RUNS: usize = 10;
const DEFAULT_ITERATIONS: usize = 100;

fn main() -> Result<(), HttpClientError> {
    let args: Vec<String> = std::env::args().collect();
    let runs = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_RUNS);
    let iterations = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_ITERATIONS);

    println!("=== HTTPS Connect Benchmark ===\n");
    println!("Runs: {}, Iterations per run: {}\n", runs, iterations);

    let mut all_run_avgs = Vec::new();

    for run in 0..runs {
        println!("Run {}/{}...", run + 1, runs);

        let mut times = Vec::with_capacity(iterations);
        let mut errors = 0;

        for i in 0..iterations {
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

            match result {
                Ok(_) => times.push(elapsed),
                Err(e) => {
                    errors += 1;
                    if errors <= 3 {
                        println!("Error {}: {:?}", errors, e);
                    }
                }
            }

            if (i + 1) % 10 == 0 {
                println!("  Completed {}/{}", i + 1, iterations);
            }
        }

        if !times.is_empty() {
            let total: std::time::Duration = times.iter().sum();
            let avg = total / times.len() as u32;
            all_run_avgs.push(avg);
            println!("  Avg: {:?}", avg);
        }
    }

    println!("\n=== Summary ===\n");
    println!("All run averages: {:?}", all_run_avgs);

    if !all_run_avgs.is_empty() {
        let overall_avg = all_run_avgs.iter().sum::<std::time::Duration>() / all_run_avgs.len() as u32;
        let mut sorted = all_run_avgs.clone();
        sorted.sort();
        let min = sorted.first().unwrap();
        let max = sorted.last().unwrap();

        println!("\nOverall Average: {:?}", overall_avg);
        println!("Min: {:?}", min);
        println!("Max: {:?}", max);
    }

    Ok(())
}
use ylong_http_client::async_impl::{Body, Client, Request};
use ylong_http_client::HttpClientError;

const TARGET_URL: &str = "https://example.com";
const DEFAULT_RUNS: usize = 10;
const DEFAULT_ITERATIONS: usize = 50;

struct StageTimes {
    dns_ms: u64,
    tcp_ms: u64,
    tls_ms: u64,
    total_ms: u64,
}

fn main() -> Result<(), HttpClientError> {
    let args: Vec<String> = std::env::args().collect();
    let runs = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_RUNS);
    let iterations = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_ITERATIONS);

    println!("=== HTTPS Connect Detailed Timing ===\n");
    println!("Runs: {}, Iterations per run: {}\n", runs, iterations);

    let mut all_dns = Vec::new();
    let mut all_tcp = Vec::new();
    let mut all_tls = Vec::new();
    let mut all_total = Vec::new();

    for run in 0..runs {
        println!("Run {}/{}...", run + 1, runs);

        let mut stage_times = Vec::with_capacity(iterations);

        for i in 0..iterations {
            let result = ylong_runtime::block_on(async {
                let client = Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()?;

                let request = Request::builder()
                    .url(TARGET_URL)
                    .body(Body::empty())?;

                let response = client.request(request).await?;

                let time_group = response.time_group();

                let dns = time_group.dns_duration()
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                let tcp = time_group.tcp_duration()
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                let tls = time_group.tls_duration()
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                Ok::<_, HttpClientError>((dns, tcp, tls))
            });

            match result {
                Ok((dns, tcp, tls)) => {
                    stage_times.push(StageTimes {
                        dns_ms: dns,
                        tcp_ms: tcp,
                        tls_ms: tls,
                        total_ms: dns + tcp + tls,
                    });
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }

            if (i + 1) % 10 == 0 {
                println!("  Completed {}/{}", i + 1, iterations);
            }
        }

        if !stage_times.is_empty() {
            let avg_dns = stage_times.iter().map(|s| s.dns_ms).sum::<u64>() / stage_times.len() as u64;
            let avg_tcp = stage_times.iter().map(|s| s.tcp_ms).sum::<u64>() / stage_times.len() as u64;
            let avg_tls = stage_times.iter().map(|s| s.tls_ms).sum::<u64>() / stage_times.len() as u64;
            let avg_total = stage_times.iter().map(|s| s.total_ms).sum::<u64>() / stage_times.len() as u64;

            all_dns.push(avg_dns);
            all_tcp.push(avg_tcp);
            all_tls.push(avg_tls);
            all_total.push(avg_total);

            println!("  Avg - DNS: {}ms, TCP: {}ms, TLS: {}ms, Total: {}ms",
                avg_dns, avg_tcp, avg_tls, avg_total);
        }
    }

    println!("\n=== Summary ===\n");

    if !all_dns.is_empty() {
        let avg = |v: &[u64]| v.iter().sum::<u64>() / v.len() as u64;
        println!("DNS avg: {}ms", avg(&all_dns));
        println!("TCP avg: {}ms", avg(&all_tcp));
        println!("TLS avg: {}ms", avg(&all_tls));
        println!("Total avg: {}ms", avg(&all_total));
    }

    Ok(())
}
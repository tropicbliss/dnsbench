mod utils;

use anyhow::Result;
use clap::Parser;
use cli_table::{format::Justify, print_stdout, Table, WithTitle};
use console::Term;
use hdrhistogram::Histogram;
use indicatif::ProgressBar;
use std::{
    collections::HashMap,
    net::IpAddr,
    num::NonZeroUsize,
    path::PathBuf,
    time::{Duration, Instant},
};
use trust_dns_client::rr::Name;
use utils::{parse_dns_addrs, resolve};

/// Simple program to benchmark DNS servers
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Dummy domain name to lookup
    #[clap(short, long)]
    domain_name: Name,

    /// Number of requests to run for each DNS server
    #[clap(short, long, default_value = "10")]
    attempts: NonZeroUsize,

    /// File containing newline delimited DNS addresses to measure
    #[clap(short, long)]
    file: PathBuf,

    /// Rate limited delay between each query of the same DNS server in seconds
    #[clap(short, long, default_value = "5")]
    rate_limit: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let dns_servers = parse_dns_addrs(args.file)?;
    let attempts = usize::from(args.attempts);
    let total_requests = dns_servers.len() * attempts;
    let mut results = Vec::with_capacity(total_requests);
    let mut last_start_times = HashMap::new();
    let rate_limit = Duration::from_secs(args.rate_limit);
    let term = Term::stdout();
    term.write_line("Benchmarking...")?;
    let progress_bar = ProgressBar::new(total_requests.try_into()?);
    let start_time = Instant::now();
    for _ in 0..attempts {
        for dns_server in &dns_servers {
            if let Some(previous_time) = last_start_times.get(dns_server) {
                let duration_from_previous_run = Instant::now().duration_since(*previous_time);
                let time_to_wait = rate_limit
                    .checked_sub(duration_from_previous_run)
                    .unwrap_or_default();
                std::thread::sleep(time_to_wait);
            }
            let elapsed = resolve(args.domain_name.clone(), dns_server.clone());
            let ended = Instant::now();
            let result = match elapsed {
                Ok(d) => ResultState::Success(d),
                Err(_) => ResultState::Failed,
            };
            let result = BenchResult { dns_server, result };
            last_start_times.insert(dns_server, ended);
            progress_bar.inc(1);
            results.push(result);
        }
    }
    let total_time_taken = start_time.elapsed();
    progress_bar.finish_and_clear();
    term.clear_last_lines(2)?;
    term.write_line(&format!("Total time taken: {:?}", total_time_taken))?;
    let mut dns_results = Vec::with_capacity(dns_servers.len());
    for dns_server in &dns_servers {
        let filter_by_dns = results.iter().filter(|r| r.dns_server == dns_server);
        let failed_requests = filter_by_dns
            .clone()
            .filter(|r| r.result == ResultState::Failed)
            .count();
        let mut hist = Histogram::<u64>::new(3).unwrap();
        for res in filter_by_dns {
            if let ResultState::Success(duration) = res.result {
                hist.record(duration.as_millis().try_into()?)?;
            }
        }
        let result = DnsResult {
            dns: dns_server,
            failed: failed_requests,
            hist,
        };
        dns_results.push(result);
    }
    dns_results.sort_unstable_by(|a, b| a.hist.mean().partial_cmp(&b.hist.mean()).unwrap());
    term.write_line("DNS servers are ordered from best to worst by its mean request, but it's best to look at the data and rank the servers yourself.")?;
    term.write_line("")?;
    render_result(dns_results, attempts)?;
    Ok(())
}

fn render_result(dns_results: Vec<DnsResult>, attempts: usize) -> Result<()> {
    let data: Vec<_> = dns_results
        .into_iter()
        .map(|dns_result| TableResult {
            dns: *dns_result.dns,
            requests: attempts,
            errors: dns_result.failed,
            min: dns_result.hist.min(),
            p50: dns_result.hist.value_at_percentile(50.0),
            p95: dns_result.hist.value_at_percentile(95.0),
            p99: dns_result.hist.value_at_percentile(99.0),
            p999: dns_result.hist.value_at_percentile(99.9),
            max: dns_result.hist.max(),
        })
        .collect();
    print_stdout(data.with_title())?;
    Ok(())
}

struct BenchResult<'a> {
    dns_server: &'a IpAddr,
    result: ResultState,
}

#[derive(PartialEq, Eq)]
enum ResultState {
    Success(Duration),
    Failed,
}

struct DnsResult<'a> {
    dns: &'a IpAddr,
    failed: usize,
    hist: Histogram<u64>,
}

#[derive(Table)]
struct TableResult {
    #[table(title = "DNS Server", justify = "Justify::Right")]
    dns: IpAddr,

    #[table(title = "Requests", justify = "Justify::Right")]
    requests: usize,

    #[table(title = "Errors", justify = "Justify::Right")]
    errors: usize,

    #[table(title = "Min (ms)", justify = "Justify::Right")]
    min: u64,

    #[table(title = "p50 (ms)", justify = "Justify::Right")]
    p50: u64,

    #[table(title = "p95 (ms)", justify = "Justify::Right")]
    p95: u64,

    #[table(title = "p99 (ms)", justify = "Justify::Right")]
    p99: u64,

    #[table(title = "p99.9 (ms)", justify = "Justify::Right")]
    p999: u64,

    #[table(title = "Max (ms)", justify = "Justify::Right")]
    max: u64,
}

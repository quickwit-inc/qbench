use std::time::Instant;

use anyhow;
use structopt::StructOpt;
use tracing::info;
use rand::prelude::*;

#[derive(Debug, Clone, StructOpt)]
struct Bench {
    #[structopt(long)]
    urls: Vec<String>,
    #[structopt(long)]
    concurrency: usize,
}

async fn run_one(url: &str) -> anyhow::Result<u64> {
    let res = reqwest::get(url).await;
    let mut num_bytes: u64 = 0;
    match res {
        Ok(mut resp) => {
            while let Some(bytes) = resp.chunk().await? {
                num_bytes += bytes.len() as u64;
            }
        }
        Err(err) => {
            tracing::error!(err=?err);
        }
    }
    Ok(num_bytes)
}

async fn run_bench(run_id: usize, mut bench: Bench) {
    loop {
        bench.urls.shuffle(&mut rand::thread_rng());
        for url in &bench.urls {
            let now = Instant::now();
            let res_one = run_one(url).await;
            let elapsed = now.elapsed();
            info!(id=run_id, res=?res_one, elapsed=?elapsed);
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let bench = Bench::from_args();
    let mut threads = Vec::new();
    for i in 0..bench.concurrency {
        let bench_clone = bench.clone();
        let handle = tokio::spawn(async move { run_bench(i, bench_clone).await });
        threads.push(handle);
    }
    for handle in threads {
        handle.await?;
    }
    Ok(())
}

use std::time::Instant;

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    let limit = (n as f64).sqrt() as u64;
    let mut d = 2u64;
    while d <= limit {
        if n % d == 0 {
            return false;
        }
        d += 1;
    }
    true
}

async fn count_primes_in_range(from: u64, to: u64) -> u32 {
    let mut count = 0u32;
    for n in from..=to {
        if is_prime(n) {
            count += 1;
        }
    }
    count
}

#[tokio::main]
async fn main() {
    let total: u64 = 10_000_000;
    let fibers: u64 = 100;
    let chunk_size = total / fibers;

    let start = Instant::now();

    let mut handles = Vec::with_capacity(fibers as usize);
    for i in 0..fibers {
        let from = i * chunk_size;
        let to = (i + 1) * chunk_size;
        handles.push(tokio::spawn(count_primes_in_range(from, to)));
    }

    let mut sum: u32 = 0;
    for h in handles {
        sum += h.await.expect("task panicked");
    }

    let elapsed_ms = start.elapsed().as_millis();
    println!(
        "Found {} primes in {}ms across {} fibers (tokio)",
        sum, elapsed_ms, fibers
    );
}

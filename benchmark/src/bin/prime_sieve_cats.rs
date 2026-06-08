use rust_cats_core::core::HKT;
use rust_cats_effect::core::IoH;
use rust_cats_effect::data::io::{IO, IoResult};
use rust_cats_effect::data::runtime::{unsafe_run_sync, worker_count};
use rust_cats_effect::for_io;
use rust_cats_effect::syntax::par_traverse;
use rust_cats_effect::typeclasses::{Clock, Sync};

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

fn count_primes_in_range<F: HKT + Sync<F, String>>(from: u64, to: u64) -> F::Applied<u32> {
    F::delay(move || {
        let mut count = 0u32;
        for n in from..to {
            if is_prime(n) {
                count += 1;
            }
        }
        count
    })
}

fn run() -> IO<()> {
    let total: u64 = 10_000_000;
    let fibers: u64 = 100;
    let chunk_size = total / fibers;

    for_io!(
        start   <- <IoH as Clock<IoH>>::monotonic();
        let chunks = (0..fibers).map(|i| (i * chunk_size, (i + 1) * chunk_size)).collect::<Vec<_>>();
        results <- par_traverse(chunks, |(from, to)| count_primes_in_range::<IoH>(from, to));
        end     <- <IoH as Clock<IoH>>::monotonic();
        let total_primes: u32 = results.iter().sum();
        let elapsed = end.saturating_sub(start).as_millis();
        IO::println(format!(
            "Found {} primes in {}ms across {} fibers, {} workers (rust-cats-effect)",
            total_primes, elapsed, fibers, worker_count()
        ));
        yield ()
    )
}

fn main() {
    match unsafe_run_sync(run()) {
        IoResult::Completed(()) => {}
        IoResult::Errored(e) => eprintln!("error: {}", e),
        IoResult::Canceled => eprintln!("canceled"),
    }
}

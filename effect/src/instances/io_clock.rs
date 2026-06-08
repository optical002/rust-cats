use crate::core::IoH;
use crate::data::io::{IO, IoResult};
use crate::data::runtime;
use crate::typeclasses::Clock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

impl Clock<IoH> for IoH {
    fn monotonic() -> IO<Duration> {
        IO::new(|_| {
            let now = Instant::now();
            let elapsed = now.saturating_duration_since(runtime::start_instant());
            IoResult::Completed(elapsed)
        })
    }

    fn realtime() -> IO<Duration> {
        IO::new(|_| {
            let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            IoResult::Completed(d)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::runtime::unsafe_run_sync;

    #[test]
    fn test_monotonic_is_nondecreasing() {
        let a = unsafe_run_sync(<IoH as Clock<IoH>>::monotonic());
        let b = unsafe_run_sync(<IoH as Clock<IoH>>::monotonic());
        match (a, b) {
            (IoResult::Completed(da), IoResult::Completed(db)) => assert!(db >= da),
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn test_realtime_is_positive() {
        match unsafe_run_sync(<IoH as Clock<IoH>>::realtime()) {
            IoResult::Completed(d) => assert!(d.as_secs() > 0),
            _ => panic!("expected Completed"),
        }
    }
}

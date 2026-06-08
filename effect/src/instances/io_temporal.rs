use crate::core::IoH;
use crate::data::io::{IO, IoResult};
use crate::data::runtime::{self, ParkSlot};
use crate::typeclasses::Temporal;
use std::sync::Arc;
use std::time::Duration;

impl Temporal<IoH, String> for IoH {
    fn sleep(d: Duration) -> IO<()> {
        IO::new(move |_| {
            let park = Arc::new(ParkSlot::new());
            let park_for_timer = park.clone();
            runtime::schedule_after(
                d,
                Box::new(move || {
                    park_for_timer.signal();
                }),
            );
            runtime::block_until(park);
            IoResult::Completed(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::runtime::unsafe_run_sync;
    use std::time::Instant;

    #[test]
    fn test_sleep_elapses_duration() {
        let start = Instant::now();
        let _ = unsafe_run_sync(<IoH as Temporal<IoH, String>>::sleep(Duration::from_millis(50)));
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(45));
    }
}

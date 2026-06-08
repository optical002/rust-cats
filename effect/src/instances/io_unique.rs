use crate::core::IoH;
use crate::data::io::{IO, IoResult};
use crate::data::runtime;
use crate::typeclasses::{Token, Unique};

impl Unique<IoH, String> for IoH {
    fn unique() -> IO<Token> {
        IO::new(|_| IoResult::Completed(Token(runtime::next_unique_token())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::runtime::unsafe_run_sync;

    #[test]
    fn test_unique_produces_distinct_tokens() {
        let a = unsafe_run_sync(<IoH as Unique<IoH, String>>::unique());
        let b = unsafe_run_sync(<IoH as Unique<IoH, String>>::unique());
        match (a, b) {
            (IoResult::Completed(ta), IoResult::Completed(tb)) => assert_ne!(ta, tb),
            _ => panic!("expected two Completed tokens"),
        }
    }
}

/// For-comprehension macro for `IO`, mirroring Scala's
/// `for { x <- ioA; y <- ioB; ... } yield expr`.
///
/// Lowers to `IO::flat_map` / `IO::map` chains. Bindings introduced earlier in
/// the comprehension are cloned when referenced later (Scala-style closure
/// captures), so binding types should be `Clone + Send + Sync + 'static`.
#[macro_export]
macro_rules! for_io {
    // Final yield.
    (yield $body:expr) => {
        $crate::data::io::IO::pure($body)
    };

    // `let name: type = expr;` — pure binding with explicit type.
    (let $name:ident : $ty:ty = $expr:expr; $($rest:tt)*) => {{
        let $name: $ty = $expr;
        $crate::for_io!($($rest)*)
    }};

    // `let name = expr;` — pure binding, no monadic context.
    (let $name:ident = $expr:expr; $($rest:tt)*) => {{
        let $name = $expr;
        $crate::for_io!($($rest)*)
    }};

    // Final bind with yield.
    ($name:ident <- $expr:expr; yield $body:expr) => {
        $crate::data::io::IO::map($expr, move |$name| $body)
    };

    // Intermediate bind.
    ($name:ident <- $expr:expr; $($rest:tt)*) => {
        $crate::data::io::IO::flat_map($expr, move |$name| {
            $crate::for_io!($($rest)*)
        })
    };

    // Sequencing: run `expr` for its effect, discard the result.
    ($expr:expr; $($rest:tt)*) => {
        $crate::data::io::IO::flat_map($expr, move |_| {
            $crate::for_io!($($rest)*)
        })
    };
}

#[cfg(test)]
mod tests {
    use crate::data::io::{IO, IoResult};
    use crate::data::runtime::unsafe_run_sync;

    #[test]
    fn test_yield_only() {
        let prog: IO<i32> = for_io!(yield 42);
        match unsafe_run_sync(prog) {
            IoResult::Completed(v) => assert_eq!(v, 42),
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn test_single_bind() {
        let prog: IO<i32> = for_io!(
            x <- IO::pure(10);
            yield x + 1
        );
        match unsafe_run_sync(prog) {
            IoResult::Completed(v) => assert_eq!(v, 11),
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn test_multi_bind() {
        let prog: IO<i32> = for_io!(
            x <- IO::pure(1);
            y <- IO::pure(2);
            z <- IO::pure(3);
            yield x + y + z
        );
        match unsafe_run_sync(prog) {
            IoResult::Completed(v) => assert_eq!(v, 6),
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn test_let_binding() {
        let prog: IO<i32> = for_io!(
            x <- IO::pure(2);
            let doubled = x * 2;
            yield doubled + 1
        );
        match unsafe_run_sync(prog) {
            IoResult::Completed(v) => assert_eq!(v, 5),
            _ => panic!("expected Completed"),
        }
    }
}

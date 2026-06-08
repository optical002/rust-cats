#[macro_export]
macro_rules! for_ {
    ($f:ty => yield $body:expr) => {
        <$f as $crate::typeclasses::Applicative<$f>>::pure($body)
    };

    ($f:ty => $name:ident <- $expr:expr; yield $body:expr) => {
        <$f as $crate::typeclasses::Functor<$f>>::fmap($expr, |$name| {
            let $name = $name;
            $body
        })
    };

    ($f:ty => $name:ident <- $expr:expr; $($rest:tt)*) => {
        <$f as $crate::typeclasses::FlatMap<$f>>::flat_map($expr, |$name| {
            $crate::for_!($f => $($rest)*)
        })
    };
}

#[cfg(test)]
mod tests {
    use crate::core::OptionH;

    #[test]
    fn test_single_bind_yields() {
        let result: Option<i32> = for_!(OptionH =>
            x <- Some(1);
            yield x + 10
        );
        assert_eq!(result, Some(11));
    }

    #[test]
    fn test_two_binds() {
        let result: Option<i32> = for_!(OptionH =>
            x <- Some(1);
            y <- Some(2);
            yield x + y
        );
        assert_eq!(result, Some(3));
    }

    #[test]
    fn test_three_binds() {
        let result: Option<i32> = for_!(OptionH =>
            x <- Some(1);
            y <- Some(2);
            z <- Some(3);
            yield x + y + z
        );
        assert_eq!(result, Some(6));
    }

    #[test]
    fn test_short_circuit_on_none() {
        let result: Option<i32> = for_!(OptionH =>
            x <- Some(1);
            _y <- Option::<i32>::None;
            yield x
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_pure_only() {
        let result: Option<i32> = for_!(OptionH => yield 42);
        assert_eq!(result, Some(42));
    }
}

#[macro_export]
macro_rules! choice {
    ( $e:expr, $f:expr, $g:expr ) => {
        ($e & $f) ^ ((!$e) & $g)
    };
}

#[macro_export]
macro_rules! median {
    ( $e:expr, $f:expr, $g:expr ) => {
        ($e & $f) | ($e & $g) | ($f & $g)
    };
}

#[macro_export]
macro_rules! rotate {
    ( $x:expr, $n:expr ) => {{
        let bits = std::mem::size_of_val(&$x) * 8;
        let shift = $n % bits;
        ($x.checked_shr(shift as u32).unwrap_or(0))
            | ($x.checked_shl((bits - shift) as u32).unwrap_or(0))
    }};
}

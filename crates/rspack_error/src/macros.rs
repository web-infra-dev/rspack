#[macro_export]
macro_rules! error {
  ($str:literal $(,)?) => {{
    $crate::Error::error(format!($str))
  }};
  ($expr:expr $(,)?) => {{
    $crate::Error::error($expr)
  }};
  ($fmt:expr, $($arg:tt)*) => {{
    let err = format!($fmt, $($arg)*);
    $crate::Error::error(err)
  }};
}

#[macro_export]
macro_rules! error_bail {
  ($str:literal $(,)?) => {
    return Err($crate::Error::error(format!($str)));
  };
  ($expr:expr $(,)?) => {
    return Err($crate::Error::error($expr));
  };
  ($fmt:expr, $($arg:tt)*) => {
    return Err($crate::Error::error(format!($fmt, $($arg)*)));
  };
}

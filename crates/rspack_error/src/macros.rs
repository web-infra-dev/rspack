#[macro_export]
macro_rules! error {
  (@base $expr:expr) => {
    $crate::Error::error($expr)
  };
  ($str:literal $(,)?) => {{
    let err = format!($str);
    $crate::error!(@base err)
  }};
  ($expr:expr $(,)?) => {{
    $crate::error!(@base $expr)
  }};
  ($fmt:expr, $($arg:tt)*) => {{
    let err = format!($fmt, $($arg)*);
    $crate::error!(@base err)
  }};
}

#[macro_export]
macro_rules! error_bail {
  ($str:literal $(,)?) => {
    return Err($crate::error!($str));
  };
  ($expr:expr $(,)?) => {
    return Err($crate::error!($expr));
  };
  ($fmt:expr, $($arg:tt)*) => {
    return Err($crate::error!($fmt, $($arg)*));
  };
}

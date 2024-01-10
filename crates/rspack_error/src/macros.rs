#[macro_export]
macro_rules! error {
  (@base $expr:expr) => {
    $crate::__private::miette!($expr)
  };
  ($str:literal $(,)?) => {{
    let err = format!($str);
    $crate::__private::error!(@base err)
  }};
  ($expr:expr $(,)?) => {{
    $crate::__private::error!(@base $expr)
  }};
  ($fmt:expr, $($arg:tt)*) => {{
    let err = format!($fmt, $($arg)*);
    $crate::__private::error!(@base err)
  }};
}

#[macro_export]
macro_rules! error_bail {
  ($str:literal $(,)?) => {
    return $crate::__private::Err($crate::error!($str));
  };
  ($expr:expr $(,)?) => {
    return $crate::__private::Err($crate::error!($expr));
  };
  ($fmt:expr, $($arg:tt)*) => {
    return $crate::__private::Err($crate::error!($fmt, $($arg)*));
  };
}

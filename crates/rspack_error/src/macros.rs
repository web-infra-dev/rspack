#[macro_export]
macro_rules! internal_error {
  (@base $expr:expr) => {
    $crate::__private::miette!($expr)
  };
  ($str:literal $(,)?) => {{
    let err = format!($str);
    $crate::__private::internal_error!(@base err)
  }};
  ($expr:expr $(,)?) => {{
    $crate::__private::internal_error!(@base $expr)
  }};
  ($fmt:expr, $($arg:tt)*) => {{
    let err = format!($fmt, $($arg)*);
    $crate::__private::internal_error!(@base err)
  }};
}

#[macro_export]
macro_rules! internal_error_bail {
  ($str:literal $(,)?) => {
    return $crate::__private::Err($crate::internal_error!($str));
  };
  ($expr:expr $(,)?) => {
    return $crate::__private::Err($crate::internal_error!($expr));
  };
  ($fmt:expr, $($arg:tt)*) => {
    return $crate::__private::Err($crate::internal_error!($fmt, $($arg)*));
  };
}

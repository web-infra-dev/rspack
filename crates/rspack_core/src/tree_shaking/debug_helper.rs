#[macro_export]
macro_rules! println_matches {
    ($id:expr, $($arg:tt)*) => {
        if ($crate::tree_shaking::debug_care_module_id($id)) {
            println!($($arg)*);
        }
    };
}
#[macro_export]
macro_rules! dbg_matches {
  ($id:expr, $val:expr $(,)?) => {
    if ($crate::tree_shaking::debug_care_module_id($id)) {
      dbg!($id,$val);
    }
  };
  ($id:expr, $($val:expr),+ $(,)?) => {
    // Use of `match` here is intentional because it affects the lifetimes
    // of temporaries - https://stackoverflow.com/a/48732525/1063961
    if ($crate::tree_shaking::debug_care_module_id($id)) {
      dbg!($id,$($val),+);
    }
  };
}

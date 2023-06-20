#![feature(box_patterns)]

macro_rules! regex {
  // `()` indicates that the macro takes no argument.
  ($pat:literal) => {
    // The macro will expand into the contents of this block.
    RspackRegex::with_flags($pat, "").expect("should compiled")
  };
  ($pat:literal, $flags:ident) => {
    // The macro will expand into the contents of this block.
    RspackRegex::with_flags($pat, stringify!($flags)).expect("should compiled")
  };
}

#[cfg(test)]
mod test_regex {

  use rspack_regex::RspackRegex;

  #[test]
  fn test_basic() {
    // should not panic

    assert!(RspackRegex::with_flags("test\\\\", "").is_ok());
  }

  #[test]
  fn case() {
    let ends_with_js = regex!("\\.js$");
    let ends_with_js_ignore_case = regex!("\\.js$", i);
    // case sensitive
    assert!(ends_with_js.test(".js"));
    assert!(!ends_with_js.test(".JS"));
    // ignore case
    assert!(ends_with_js_ignore_case.test(".js"));
    assert!(ends_with_js_ignore_case.test(".JS"));
  }
}

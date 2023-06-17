#![feature(box_patterns)]

#[cfg(test)]
mod test_regex {

  use rspack_regex::RspackRegex;

  #[test]
  fn test_basic() {
    // should not panic

    assert!(RspackRegex::with_flags("test\\\\", "").is_ok());
  }
}

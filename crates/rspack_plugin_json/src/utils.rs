pub fn escape_json(json_str: &str) -> String {
  use std::fmt::Write;

  let mut escaped = String::with_capacity(json_str.len());

  for c in json_str.chars() {
    if c == '\u{2028}' {
      write!(&mut escaped, "\\\\u{:04X}", &0x2028).unwrap();
    } else if c == '\u{2029}' {
      write!(&mut escaped, "\\\\u{:04X}", &0x2029).unwrap();
    } else {
      escaped.push(c)
    }
  }

  escaped
}

mod test {
  #[test]
  fn test_escape_json() {
    let cases = vec![
      (
        r#"{"LS":" ","PS":" ","escaped":"\\u2028"}"#,
        r#"{"LS":"\\u2028","PS":"\\u2029","escaped":"\\u2028"}"#,
      ),
      (r#"{"na\ me": "\ntest"}"#, r#"{"na\ me": "\ntest"}"#),
      (r#"{"a": \n\r\t"a"}"#, r#"{"a": \n\r\t"a"}"#),
      (
        r#"{"\"\\\/\b \f\t\r\n": "\"\\\/\b\f\t \r\n"}"#,
        r#"{"\"\\\/\b\\u2028\f\t\r\n": "\"\\\/\b\f\t\\u2028\r\n"}"#,
      ),
    ];

    for (source, expected) in cases {
      let escaped = super::escape_json(source);
      assert_eq!(escaped, expected)
    }
  }
}

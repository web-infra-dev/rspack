use std::borrow::Cow;

const ESCAPE_STRING_LENGTH: usize = 3;
const U2028: &str = "\\u2028";
const U2029: &str = "\\u2029";

pub fn escape_json(input: &str) -> Cow<'_, str> {
  let mut vec = input
    .match_indices('\u{2028}')
    .chain(input.match_indices('\u{2029}'))
    .collect::<Vec<_>>();

  if !vec.is_empty() {
    // The replace algorithm require our byte index is ordered,
    // which we could not ensure in the two chaining `match_indices`,
    // because `\u2028` and `\u2029` could place in any where in a string.
    //  We need to sort the indices to make sure the byte index is ordered.
    // ## Example
    // ```json
    // "\u2028\u2029\u2028"
    // ```
    // The final vec would be `[(0, '\u{2028}'),(2, '\u{2028}'),(1, '\u{2029}'),]`
    vec.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    // Length of `\u2028` and `\u2029` are both 3 , and would be replaced by raw string `r#"\\u2028"#` or `r#"\\u2029"#`
    // which length is 7.That's why we need to allocate extra `vec.len() * (7 - 3)` bytes.
    let mut ret = String::with_capacity(input.len() + vec.len() * 4);
    let mut last = 0;
    // The replace algorithm is copied from rust std lib https://doc.rust-lang.org/src/alloc/str.rs.html#288
    for (i, str) in vec {
      ret.push_str(unsafe { input.get_unchecked(last..i) });
      ret.push_str(if str == "\u{2028}" { U2028 } else { U2029 });
      last = i + ESCAPE_STRING_LENGTH;
    }
    ret.push_str(unsafe { input.get_unchecked(last..) });
    Cow::Owned(ret)
  } else {
    Cow::Borrowed(input)
  }
}
mod test {
  #[test]
  fn test_escape_json() {
    let cases = vec![
      (
        r#"{"LS":" ","PS":" ","escaped":"\\u2028"}"#,
        r#"{"LS":"\u2028","PS":"\u2029","escaped":"\\u2028"}"#,
      ),
      (r#"[" "," ","\\u2028"]"#, r#"["\u2029","\u2028","\\u2028"]"#),
      (r#"{"na\ me": "\ntest"}"#, r#"{"na\ me": "\ntest"}"#),
      (r#"{"a": \n\r\t"a"}"#, r#"{"a": \n\r\t"a"}"#),
      (
        r#"{"\"\\\/\b \f\t\r\n": "\"\\\/\b\f\t \r\n"}"#,
        r#"{"\"\\\/\b\u2028\f\t\r\n": "\"\\\/\b\f\t\u2028\r\n"}"#,
      ),
    ];

    for (source, expected) in cases {
      let escaped = super::escape_json(source);
      assert_eq!(escaped, expected)
    }
  }
}

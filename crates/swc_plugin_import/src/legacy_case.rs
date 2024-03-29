use core::fmt;

// for compatibility with babel_plugin_import versions before 1.13.7 which is its widely used versions.
// [<1.13.7]{@link https://github.com/umijs/babel-plugin-import/blob/8efa0aa1472f0e6c1d574eb04bf2b9100fccf5a7/src/Plugin.js#L4-L6}
//     only transform uppercase characters to a hyphen followed by its lowercase
// [1.13.8]{@link https://github.com/umijs/babel-plugin-import/blob/8cc97c8e394891a684d7300d26cbe2b86f20d076/src/Plugin.js#L4-L10}
//     transform like `kebabCase`
fn transform_like_babel_plugin_import(s: &str, sym: &str, f: &mut fmt::Formatter) -> fmt::Result {
  let char_indices = s.char_indices().peekable();
  let mut is_first = true;
  for (_, c) in char_indices {
    if c.is_uppercase() {
      if is_first {
        write!(f, "{}", c.to_lowercase())?;
      } else {
        write!(f, "{}{}", sym, c.to_lowercase())?;
      }
    } else {
      write!(f, "{}", c)?;
    }
    is_first = false
  }
  Ok(())
}

pub struct AsLegacyKebabCase<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> fmt::Display for AsLegacyKebabCase<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    transform_like_babel_plugin_import(self.0.as_ref(), "-", f)
  }
}

pub struct AsLegacySnakeCase<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> fmt::Display for AsLegacySnakeCase<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    transform_like_babel_plugin_import(self.0.as_ref(), "_", f)
  }
}

// transform an identifier like babel_plugin_import@<1.13.7 did.
// it's like `kebabCase` but differs, e.g.,
// ```rust
// assert_eq!(
//   identifier_to_legacy_kebab_case("HTTPRequest"),
//   "h-t-t-p-request"
// );
// ```
// it won't be `impl` for `str` as it's not strict name convention
pub fn identifier_to_legacy_kebab_case(s: &str) -> String {
  AsLegacyKebabCase(s).to_string()
}

// transform an identifier like babel_plugin_import@<1.13.7 did with `camel2UnderlineComponentName`.
// it's like `kebabCase` but differs, e.g.,
// ```rust
// assert_eq!(
//   identifier_to_legacy_snake_case("HTTPRequest"),
//   "h_t_t_p_request"
// );
// ```
// it won't be `impl` for `str` as it's not strict name convention
pub fn identifier_to_legacy_snake_case(s: &str) -> String {
  AsLegacySnakeCase(s).to_string()
}

#[test]
fn test_legacy_case() {
  let cases = [
    ("XIPObject", "x-i-p-object", "x_i_p_object"),
    ("XMLHttpRequest", "x-m-l-http-request", "x_m_l_http_request"),
    ("PascalCase", "pascal-case", "pascal_case"),
    ("snake_case", "snake_case", "snake_case"),
    ("WithNumber3d", "with-number3d", "with_number3d"),
  ];

  for (src, legacy_kebab, legacy_snake) in cases {
    assert_eq!(identifier_to_legacy_kebab_case(src), legacy_kebab);
    assert_eq!(identifier_to_legacy_snake_case(src), legacy_snake);
  }
}

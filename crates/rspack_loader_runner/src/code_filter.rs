use memchr::memmem;
use rspack_cacheable::cacheable;
use rspack_regex::RspackRegex;

#[cacheable]
#[derive(Debug)]
pub enum CodeFilter {
  String(String),
  Regexp(RspackRegex),
}

impl CodeFilter {
  pub fn matches(&self, code: &str) -> bool {
    match self {
      Self::String(pattern) => memmem::find(code.as_bytes(), pattern.as_bytes()).is_some(),
      Self::Regexp(pattern) => pattern.test(code),
    }
  }
}

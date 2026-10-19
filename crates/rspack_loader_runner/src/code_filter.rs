use memchr::memmem;
use rspack_cacheable::cacheable;
use rspack_hash::RspackHasher;
use rspack_regex::RspackRegex;

#[cacheable]
#[derive(Debug)]
pub enum CodeFilter {
  String(String),
  Regexp(RspackRegex),
}

impl CodeFilter {
  pub(crate) fn update_fingerprint(&self, hasher: &mut RspackHasher) {
    fn write_part(hasher: &mut RspackHasher, value: &str) {
      hasher.write(&(value.len() as u64).to_be_bytes());
      hasher.write(value.as_bytes());
    }

    match self {
      Self::String(value) => {
        hasher.write(b"string");
        write_part(hasher, value);
      }
      Self::Regexp(value) => {
        hasher.write(b"regexp");
        write_part(hasher, value.source());
        write_part(hasher, value.flags());
      }
    }
  }

  pub fn matches(&self, code: &str) -> bool {
    match self {
      Self::String(pattern) => memmem::find(code.as_bytes(), pattern.as_bytes()).is_some(),
      Self::Regexp(pattern) => pattern.test(code),
    }
  }
}

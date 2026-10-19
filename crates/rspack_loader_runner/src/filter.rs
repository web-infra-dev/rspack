use rspack_cacheable::cacheable;
use rspack_hash::{HashDigest, HashFunction, RspackHashDigest, RspackHasher};

use crate::CodeFilter;

#[cacheable]
#[derive(Debug)]
pub struct LoaderFilter {
  code: Option<CodeFilter>,
  fingerprint: RspackHashDigest,
}

impl LoaderFilter {
  pub fn new(code: Option<CodeFilter>) -> Self {
    let mut hasher = RspackHasher::new(&HashFunction::SHA256);
    hasher.write(b"loader-filter-v1");
    if let Some(code) = &code {
      code.update_fingerprint(&mut hasher);
    } else {
      hasher.write(b"none");
    }
    Self {
      code,
      fingerprint: hasher.digest(&HashDigest::Hex),
    }
  }

  pub fn fingerprint(&self) -> &str {
    self.fingerprint.encoded()
  }

  pub fn code(&self) -> Option<&CodeFilter> {
    self.code.as_ref()
  }

  pub fn matches(&self, content: &str) -> bool {
    self.code.as_ref().is_none_or(|code| code.matches(content))
  }
}

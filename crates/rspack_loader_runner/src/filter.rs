use rspack_cacheable::cacheable;

use crate::CodeFilter;

#[cacheable]
#[derive(Debug)]
pub struct LoaderFilter {
  code: Option<CodeFilter>,
}

impl LoaderFilter {
  pub fn new(code: Option<CodeFilter>) -> Self {
    Self { code }
  }

  pub fn code(&self) -> Option<&CodeFilter> {
    self.code.as_ref()
  }

  pub fn matches(&self, content: &str) -> bool {
    self.code.as_ref().is_none_or(|code| code.matches(content))
  }
}

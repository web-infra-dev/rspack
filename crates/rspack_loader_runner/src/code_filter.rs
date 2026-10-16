use rspack_cacheable::{
  ContextGuard, Result as CacheableResult, cacheable,
  with::{Custom, CustomConverter},
};
use rspack_regex::RspackRegex;

#[cacheable]
#[doc(hidden)]
pub struct SerializableCodeFilter {
  include: Option<RspackRegex>,
  exclude: Option<RspackRegex>,
}

#[cacheable(with=Custom)]
#[derive(Debug)]
pub struct CodeFilter {
  include: Option<RspackRegex>,
  exclude: Option<RspackRegex>,
}

impl CodeFilter {
  pub fn new(include: Option<RspackRegex>, exclude: Option<RspackRegex>) -> Self {
    Self { include, exclude }
  }

  pub fn include(&self) -> Option<&RspackRegex> {
    self.include.as_ref()
  }

  pub fn exclude(&self) -> Option<&RspackRegex> {
    self.exclude.as_ref()
  }

  pub fn matches(&self, code: &str) -> bool {
    if self
      .exclude
      .as_ref()
      .is_some_and(|exclude| exclude.test(code))
    {
      return false;
    }
    self
      .include
      .as_ref()
      .is_none_or(|include| include.test(code))
  }
}

impl CustomConverter for CodeFilter {
  type Target = SerializableCodeFilter;

  fn serialize(&self, _guard: &ContextGuard) -> CacheableResult<Self::Target> {
    Ok(SerializableCodeFilter {
      include: self.include.clone(),
      exclude: self.exclude.clone(),
    })
  }

  fn deserialize(data: Self::Target, _guard: &ContextGuard) -> CacheableResult<Self> {
    Ok(Self::new(data.include, data.exclude))
  }
}

use rspack_cacheable::{
  ContextGuard, Result, cacheable,
  with::{AsConverter, AsStringConverter},
};
use rspack_regex::RspackRegex;

#[cacheable]
pub struct RegexRestriction(String);

impl AsConverter<RspackRegex> for RegexRestriction {
  fn serialize(data: &RspackRegex, guard: &ContextGuard) -> Result<Self> {
    // Escaped paths in regular expressions cannot be safely rebased or detected by byte scans.
    guard.mark_outside_root();
    Ok(Self(data.to_string()?))
  }

  fn deserialize(self, _guard: &ContextGuard) -> Result<RspackRegex> {
    RspackRegex::from_str(&self.0)
  }
}

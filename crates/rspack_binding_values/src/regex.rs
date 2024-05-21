use napi_derive::napi;
use rspack_regex::RspackRegex;

#[derive(Debug)]
#[napi(object)]
pub struct RawRegex {
  pub source: String,
  pub flags: String,
}

impl From<RspackRegex> for RawRegex {
  fn from(value: RspackRegex) -> Self {
    Self {
      source: value.source().to_string(),
      flags: value.flags().to_string(),
    }
  }
}

impl TryFrom<RawRegex> for RspackRegex {
  type Error = rspack_error::Error;

  fn try_from(value: RawRegex) -> Result<Self, Self::Error> {
    Self::with_flags(&value.source, &value.flags)
  }
}

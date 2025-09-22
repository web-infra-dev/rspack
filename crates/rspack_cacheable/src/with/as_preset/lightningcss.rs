use lightningcss::targets::Browsers;
use rkyv::{
  Place,
  rancor::Fallible,
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use super::AsPreset;
use crate::{DeserializeError, SerializeError};

pub struct BrowsersResolver {
  inner: StringResolver,
  value: String,
}

// Helper functions for custom serialization without serde
fn browsers_to_string(browsers: &Browsers) -> String {
  let parts = vec![
    browsers.android.map(|v| format!("android:{}", v)),
    browsers.chrome.map(|v| format!("chrome:{}", v)),
    browsers.edge.map(|v| format!("edge:{}", v)),
    browsers.firefox.map(|v| format!("firefox:{}", v)),
    browsers.ie.map(|v| format!("ie:{}", v)),
    browsers.ios_saf.map(|v| format!("ios_saf:{}", v)),
    browsers.opera.map(|v| format!("opera:{}", v)),
    browsers.safari.map(|v| format!("safari:{}", v)),
    browsers.samsung.map(|v| format!("samsung:{}", v)),
  ];

  parts.into_iter().flatten().collect::<Vec<_>>().join(",")
}

fn string_to_browsers(s: &str) -> Result<Browsers, DeserializeError> {
  let mut browsers = Browsers::default();

  if s.is_empty() {
    return Ok(browsers);
  }

  for part in s.split(',') {
    let mut split = part.split(':');
    let browser = split
      .next()
      .ok_or_else(|| DeserializeError::MessageError("invalid browser format"))?;
    let version_str = split
      .next()
      .ok_or_else(|| DeserializeError::MessageError("invalid browser format"))?;

    let version = version_str
      .parse::<u32>()
      .map_err(|_| DeserializeError::MessageError("invalid version format"))?;

    match browser {
      "android" => browsers.android = Some(version),
      "chrome" => browsers.chrome = Some(version),
      "edge" => browsers.edge = Some(version),
      "firefox" => browsers.firefox = Some(version),
      "ie" => browsers.ie = Some(version),
      "ios_saf" => browsers.ios_saf = Some(version),
      "opera" => browsers.opera = Some(version),
      "safari" => browsers.safari = Some(version),
      "samsung" => browsers.samsung = Some(version),
      _ => return Err(DeserializeError::MessageError("unknown browser")),
    }
  }

  Ok(browsers)
}

impl ArchiveWith<Browsers> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = BrowsersResolver;

  #[inline]
  fn resolve_with(_field: &Browsers, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let BrowsersResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, inner, out);
  }
}

impl<S> SerializeWith<Browsers, S> for AsPreset
where
  S: Fallible<Error = SerializeError> + Writer,
{
  #[inline]
  fn serialize_with(
    field: &Browsers,
    serializer: &mut S,
  ) -> Result<Self::Resolver, SerializeError> {
    let value = browsers_to_string(field);
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(BrowsersResolver { value, inner })
  }
}

impl<D> DeserializeWith<ArchivedString, Browsers, D> for AsPreset
where
  D: Fallible<Error = DeserializeError>,
{
  fn deserialize_with(
    field: &ArchivedString,
    _deserializer: &mut D,
  ) -> Result<Browsers, DeserializeError> {
    string_to_browsers(field.as_str())
  }
}

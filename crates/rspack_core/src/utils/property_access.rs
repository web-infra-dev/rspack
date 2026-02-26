use crate::utils::property_name::{RESERVED_IDENTIFIER, SAFE_IDENTIFIER};

fn render_property_access(result: &mut String, property: &str, optional: bool) {
  let prefix = if optional { "?." } else { "." };
  if SAFE_IDENTIFIER.is_match(property) && !RESERVED_IDENTIFIER.contains(property) {
    result.push_str(&format!("{prefix}{property}"));
  } else {
    let quoted = serde_json::to_string(property).expect("should render property");
    if optional {
      result.push_str(&format!("?.[{quoted}]"));
    } else {
      result.push_str(&format!("[{quoted}]"));
    }
  }
}

pub fn property_access<S: AsRef<str>>(o: impl IntoIterator<Item = S>, start: usize) -> String {
  o.into_iter()
    .skip(start)
    .fold(String::default(), |mut s, p| {
      render_property_access(&mut s, p.as_ref(), false);
      s
    })
}

pub fn property_access_with_optional<S: AsRef<str>>(
  properties: impl IntoIterator<Item = S>,
  optionals: &[bool],
  start: usize,
) -> String {
  properties
    .into_iter()
    .skip(start)
    .enumerate()
    .fold(String::default(), |mut s, (i, p)| {
      let optional = optionals.get(i + start).copied().unwrap_or(false);
      render_property_access(&mut s, p.as_ref(), optional);
      s
    })
}

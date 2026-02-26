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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_property_access_basic() {
    assert_eq!(property_access(["a", "b", "c"], 0), ".a.b.c");
  }

  #[test]
  fn test_property_access_with_start() {
    assert_eq!(property_access(["a", "b", "c"], 1), ".b.c");
  }

  #[test]
  fn test_property_access_special_chars() {
    assert_eq!(property_access(["a-b", "c d"], 0), "[\"a-b\"][\"c d\"]");
  }

  #[test]
  fn test_property_access_reserved_words() {
    assert_eq!(
      property_access(["class", "default"], 0),
      "[\"class\"][\"default\"]"
    );
  }

  #[test]
  fn test_property_access_with_optional_all_required() {
    let props = vec!["a", "b", "c"];
    let optionals = vec![false, false, false];
    assert_eq!(
      property_access_with_optional(props, &optionals, 0),
      ".a.b.c"
    );
  }

  #[test]
  fn test_property_access_with_optional_all_optional() {
    let props = vec!["a", "b", "c"];
    let optionals = vec![true, true, true];
    assert_eq!(
      property_access_with_optional(props, &optionals, 0),
      "?.a?.b?.c"
    );
  }

  #[test]
  fn test_property_access_with_optional_mixed() {
    let props = vec!["a", "b", "c"];
    let optionals = vec![false, true, false];
    assert_eq!(
      property_access_with_optional(props, &optionals, 0),
      ".a?.b.c"
    );
  }

  #[test]
  fn test_property_access_with_optional_special_chars() {
    let props = vec!["a-b", "c"];
    let optionals = vec![true, false];
    assert_eq!(
      property_access_with_optional(props, &optionals, 0),
      "?.[\"a-b\"].c"
    );
  }

  #[test]
  fn test_property_access_with_optional_with_start() {
    let props = vec!["a", "b", "c"];
    let optionals = vec![false, true, true];
    assert_eq!(
      property_access_with_optional(props, &optionals, 1),
      "?.b?.c"
    );
  }

  #[test]
  fn test_property_access_with_optional_empty_optionals() {
    let props = vec!["a", "b"];
    let optionals: Vec<bool> = vec![];
    assert_eq!(property_access_with_optional(props, &optionals, 0), ".a.b");
  }
}

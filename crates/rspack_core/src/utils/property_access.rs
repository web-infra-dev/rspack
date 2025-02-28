use crate::utils::property_name::{RESERVED_IDENTIFIER, SAFE_IDENTIFIER};

pub fn property_access<S: AsRef<str>>(o: impl IntoIterator<Item = S>, start: usize) -> String {
  o.into_iter()
    .skip(start)
    .fold(String::default(), |mut str, property| {
      let property = property.as_ref();
      if SAFE_IDENTIFIER.is_match(property) && !RESERVED_IDENTIFIER.contains(property) {
        str.push_str(format!(".{property}").as_str());
      } else {
        str.push_str(
          format!(
            "[{}]",
            serde_json::to_string(property).expect("should render property")
          )
          .as_str(),
        );
      }
      str
    })
}

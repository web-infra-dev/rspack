use crate::utils::property_name::{RESERVED_IDENTIFIER, is_safe_identifier};

pub fn property_access<S: AsRef<str>>(o: impl IntoIterator<Item = S>, start: usize) -> String {
  o.into_iter()
    .skip(start)
    .fold(String::default(), |mut str, property| {
      let property = property.as_ref();
      if is_safe_identifier(property) && !RESERVED_IDENTIFIER.contains(property) {
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

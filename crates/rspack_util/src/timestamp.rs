use serde::{Deserialize, Serialize};

/// A Unix timestamp in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp(u64);

impl Timestamp {
  pub fn as_millis(self) -> u64 {
    self.0
  }
}

impl From<u64> for Timestamp {
  fn from(value: u64) -> Self {
    Self(value)
  }
}

impl Serialize for Timestamp {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.0.to_string())
  }
}

impl<'de> Deserialize<'de> for Timestamp {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    use serde::de::Error;

    String::deserialize(deserializer)?
      .parse()
      .map(Self)
      .map_err(D::Error::custom)
  }
}

#[cfg(test)]
mod tests {
  use super::Timestamp;

  #[test]
  fn serializes_as_string() {
    let timestamp = Timestamp::from(1_785_292_861_682);
    let value = serde_json::to_value(timestamp).unwrap();

    assert_eq!(value, "1785292861682");
    assert_eq!(
      serde_json::from_value::<Timestamp>(value).unwrap(),
      timestamp
    );
  }
}

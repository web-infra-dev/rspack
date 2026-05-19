use rspack_paths::Utf8PathBuf;

pub(super) enum ValueType {
  Undefined,
  Atom,
  Extend,
  Other,
}

pub(super) trait GetValueType {
  fn get_value_type(&self) -> ValueType;
}

impl<T: GetValueType> GetValueType for Option<T> {
  fn get_value_type(&self) -> ValueType {
    match self {
      Some(value) => value.get_value_type(),
      None => ValueType::Undefined,
    }
  }
}

impl GetValueType for Vec<String> {
  fn get_value_type(&self) -> ValueType {
    if self.iter().any(|s| s.as_str() == "...") {
      ValueType::Extend
    } else {
      ValueType::Atom
    }
  }
}

macro_rules! get_value_type_for_basic {
  ($t: ty) => {
    impl GetValueType for $t {
      fn get_value_type(&self) -> ValueType {
        ValueType::Atom
      }
    }
  };
}

get_value_type_for_basic!(bool);
get_value_type_for_basic!(Utf8PathBuf);

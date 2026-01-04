use std::path::PathBuf;

use rspack_cacheable::{
  Error, enable_cacheable as cacheable,
  with::{AsString, AsStringConverter},
};

#[cacheable(with=AsString)]
#[derive(Debug, PartialEq, Eq)]
struct Regex {
  source: String,
  flags: String,
}
impl AsStringConverter for Regex {
  fn to_string(&self) -> Result<String, Error> {
    Ok(format!("{}#{}", self.flags, self.source))
  }
  fn from_str(s: &str) -> Result<Self, Error>
  where
    Self: Sized,
  {
    let (flags, source) = s.split_once('#').expect("should have flags");
    Ok(Self {
      source: String::from(source),
      flags: String::from(flags),
    })
  }
}

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  #[cacheable(with=AsString)]
  path: PathBuf,
  #[cacheable(with=AsString)]
  regex: Regex,
}

#[test]
fn test_as_string() {
  let module = Module {
    path: PathBuf::from("/root"),
    regex: Regex {
      source: String::from("/.*/"),
      flags: String::from("g"),
    },
  };

  let bytes = rspack_cacheable::to_bytes(&module, &()).unwrap();
  let new_module: Module = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(module, new_module);
}

use rspack_cacheable::{
  cacheable,
  with::{AsRefStr, AsRefStrConverter},
};

#[derive(Debug, PartialEq, Eq)]
struct UnCacheable;

#[derive(Debug, PartialEq, Eq)]
struct Cat {
  name: String,
  uncacheable: UnCacheable,
}
impl AsRefStrConverter for Cat {
  fn as_str(&self) -> &str {
    &self.name
  }
  fn from_str(s: &str) -> Self {
    Self {
      name: String::from(s),
      uncacheable: UnCacheable,
    }
  }
}

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Home {
  #[cacheable(with=AsRefStr)]
  cat: Cat,
}

#[test]
fn test_as_ref_string() {
  let home = Home {
    cat: Cat {
      name: String::from("a"),
      uncacheable: UnCacheable,
    },
  };
  let bytes = rspack_cacheable::to_bytes(&home, &()).unwrap();
  let new_home: Home = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(home, new_home);
}

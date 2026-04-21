use rspack_cacheable::{
  CacheableContext, enable_cacheable as cacheable, enable_cacheable_dyn as cacheable_dyn,
  from_bytes, to_bytes,
};

struct Context;
impl CacheableContext for Context {
  fn project_root(&self) -> Option<&std::path::Path> {
    None
  }
}

#[cacheable_dyn]
trait Animal {
  fn color(&self) -> &str;
  fn name(&self) -> &str;
}

#[cacheable]
struct Dog {
  color: String,
}
#[cacheable_dyn]
impl Animal for Dog {
  fn color(&self) -> &str {
    &self.color
  }
  fn name(&self) -> &str {
    "dog"
  }
}

#[cacheable]
struct Cat {
  color: String,
}
#[cacheable_dyn]
impl Animal for Cat {
  fn color(&self) -> &str {
    &self.color
  }
  fn name(&self) -> &str {
    "cat"
  }
}

#[cacheable]
struct Data {
  animal: Box<dyn Animal>,
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cacheable_dyn_macro() {
  let dog_data = Data {
    animal: Box::new(Dog {
      color: String::from("black"),
    }),
  };
  assert_eq!(dog_data.animal.name(), "dog");
  assert_eq!(dog_data.animal.color(), "black");
  let ctx = Context {};
  let bytes = to_bytes(&dog_data, &ctx).unwrap();
  let deserialize_data = from_bytes::<Data, Context>(&bytes, &ctx).unwrap();
  assert_eq!(deserialize_data.animal.name(), "dog");
  assert_eq!(deserialize_data.animal.color(), "black");

  let cat_data = Data {
    animal: Box::new(Cat {
      color: String::from("white"),
    }),
  };
  assert_eq!(cat_data.animal.name(), "cat");
  assert_eq!(cat_data.animal.color(), "white");
  let ctx = Context {};
  let bytes = to_bytes(&cat_data, &ctx).unwrap();
  let deserialize_data = from_bytes::<Data, Context>(&bytes, &ctx).unwrap();
  assert_eq!(deserialize_data.animal.name(), "cat");
  assert_eq!(deserialize_data.animal.color(), "white");
}

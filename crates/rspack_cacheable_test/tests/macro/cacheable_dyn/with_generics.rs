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
trait Animal<T = ()>: Send + Sync
where
  T: Send,
{
  fn color(&self) -> &str;
  fn name(&self) -> T;
}

#[cacheable]
struct Dog {
  color: String,
}
#[cacheable_dyn]
impl Animal<&'static str> for Dog {
  fn color(&self) -> &str {
    &self.color
  }
  fn name(&self) -> &'static str {
    "dog"
  }
}

#[cacheable]
struct Cat {
  color: String,
}
#[cacheable_dyn]
impl Animal<String> for Cat {
  fn color(&self) -> &str {
    &self.color
  }
  fn name(&self) -> String {
    String::from("cat")
  }
}

#[cacheable]
struct Data {
  animal_1: Box<dyn Animal<&'static str>>,
  animal_2: Box<dyn Animal<String>>,
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cacheable_dyn_macro_with_generics() {
  let data = Data {
    animal_1: Box::new(Dog {
      color: String::from("black"),
    }),
    animal_2: Box::new(Cat {
      color: String::from("white"),
    }),
  };
  assert_eq!(data.animal_1.name(), "dog");
  assert_eq!(data.animal_1.color(), "black");
  assert_eq!(data.animal_2.name(), "cat");
  assert_eq!(data.animal_2.color(), "white");
  let ctx = Context {};
  let bytes = to_bytes(&data, &ctx).unwrap();
  let deserialize_data = from_bytes::<Data, Context>(&bytes, &ctx).unwrap();
  assert_eq!(deserialize_data.animal_1.name(), "dog");
  assert_eq!(deserialize_data.animal_1.color(), "black");
  assert_eq!(deserialize_data.animal_2.name(), "cat");
  assert_eq!(deserialize_data.animal_2.color(), "white");
}

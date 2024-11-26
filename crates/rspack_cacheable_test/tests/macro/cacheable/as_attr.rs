use rspack_cacheable::{
  cacheable, cacheable_dyn, from_bytes, to_bytes,
  with::{AsOption, AsTuple2, AsVec, Inline},
};

#[cacheable_dyn]
trait Module {}

#[cacheable]
struct NormalModule {
  inner: String,
}

#[cacheable_dyn]
impl Module for NormalModule {}

#[cacheable]
struct Data {
  block1: String,
  block2: Vec<(String, Option<String>)>,
  block3: Box<dyn Module>,
}

#[cacheable(as=Data)]
struct DataRef<'a> {
  #[cacheable(with=Inline)]
  block1: &'a String,
  #[cacheable(with=AsVec<AsTuple2<Inline, AsOption<Inline>>>)]
  block2: Vec<(&'a String, Option<&'a String>)>,
  #[allow(clippy::borrowed_box)]
  #[cacheable(with=Inline)]
  block3: &'a Box<dyn Module>,
}

#[test]
#[cfg_attr(miri, ignore)]
fn as_attr() {
  let a = Data {
    block1: "abc".into(),
    block2: vec![
      ("key1".into(), None),
      ("key2".into(), Some("value2".into())),
      ("key3".into(), Some("value3".into())),
    ],
    block3: Box::new(NormalModule {
      inner: "inner".into(),
    }),
  };
  let a_ref = DataRef {
    block1: &a.block1,
    block2: a
      .block2
      .iter()
      .map(|(key, value)| (key, value.as_ref()))
      .collect(),
    block3: &a.block3,
  };
  let bytes = to_bytes(&a, &()).unwrap();
  let bytes_ref = to_bytes(&a_ref, &()).unwrap();
  assert_eq!(bytes, bytes_ref);
  from_bytes::<Data, ()>(&bytes, &()).unwrap();
}

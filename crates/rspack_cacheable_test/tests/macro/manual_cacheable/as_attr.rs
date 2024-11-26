use rspack_cacheable::{
  cacheable_dyn, from_bytes, to_bytes,
  with::{AsOption, AsTuple2, AsVec, Inline},
};

#[cacheable_dyn]
trait Module {}

#[derive(
  rspack_cacheable::__private::rkyv::Archive,
  rspack_cacheable::__private::rkyv::Deserialize,
  rspack_cacheable::__private::rkyv::Serialize,
)]
#[rkyv(crate=rspack_cacheable::__private::rkyv)]
struct NormalModule {
  inner: String,
}

#[cacheable_dyn]
impl Module for NormalModule {}

#[derive(
  rspack_cacheable::__private::rkyv::Archive,
  rspack_cacheable::__private::rkyv::Deserialize,
  rspack_cacheable::__private::rkyv::Serialize,
)]
#[rkyv(crate=rspack_cacheable::__private::rkyv)]
struct Data {
  block1: String,
  block2: Vec<(String, Option<String>)>,
  block3: Box<dyn Module>,
}

#[derive(
  rspack_cacheable::__private::rkyv::Archive, rspack_cacheable::__private::rkyv::Serialize,
)]
#[rkyv(crate=rspack_cacheable::__private::rkyv,
         as=rspack_cacheable::__private::rkyv::Archived<Data>)]
#[rkyv(serialize_bounds(
    __S: rspack_cacheable::__private::rkyv::ser::Writer + rspack_cacheable::__private::rkyv::ser::Allocator + rspack_cacheable::__private::rkyv::rancor::Fallible<Error = rspack_cacheable::SerializeError>,
    Inline: rspack_cacheable::__private::rkyv::with::SerializeWith<&'a String, __S>,
    AsVec<AsTuple2<Inline, AsOption<Inline>>>: rspack_cacheable::__private::rkyv::with::SerializeWith<Vec<(&'a String, Option<&'a String>)>, __S>,
    Inline: rspack_cacheable::__private::rkyv::with::SerializeWith<&'a Box<dyn Module>, __S>
  ))]
struct DataRef<'a> {
  #[rkyv(omit_bounds)]
  #[rkyv(with=Inline)]
  block1: &'a String,
  #[rkyv(omit_bounds)]
  #[rkyv(with=AsVec<AsTuple2<Inline, AsOption<Inline>>>)]
  block2: Vec<(&'a String, Option<&'a String>)>,
  #[allow(clippy::borrowed_box)]
  #[rkyv(omit_bounds)]
  #[rkyv(with=Inline)]
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

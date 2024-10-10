use std::sync::Arc;

use rspack_cacheable::{
  CacheableDeserializer, DeserializeError,
  __private::rkyv::{
    rancor::Fallible,
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Place,
  },
  get_deserializer_context,
};

use super::CacheContext;
use crate::CompilerOptions;

pub struct FromContext;

impl<F> ArchiveWith<F> for FromContext {
  type Archived = ();
  type Resolver = ();

  #[inline]
  fn resolve_with(_: &F, _: Self::Resolver, _: Place<Self::Archived>) {}
}

impl<F, S: Fallible + ?Sized> SerializeWith<F, S> for FromContext {
  #[inline]
  fn serialize_with(_: &F, _: &mut S) -> Result<Self::Resolver, S::Error> {
    Ok(())
  }
}

impl DeserializeWith<(), Arc<CompilerOptions>, CacheableDeserializer> for FromContext {
  fn deserialize_with(
    _: &(),
    d: &mut CacheableDeserializer,
  ) -> Result<Arc<CompilerOptions>, DeserializeError> {
    let ctx = get_deserializer_context::<CacheContext>(d).expect("should have CacheContext");
    Ok(ctx.options.clone())
  }
}

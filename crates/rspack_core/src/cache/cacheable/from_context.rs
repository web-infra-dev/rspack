use std::sync::Arc;

use rspack_cacheable::__private::rkyv::{
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Fallible,
};
use rspack_cacheable::{CacheableDeserializer, DeserializeError};

use super::CacheContext;
use crate::CompilerOptions;

pub struct FromContext;

impl<F> ArchiveWith<F> for FromContext {
  type Archived = ();
  type Resolver = ();

  unsafe fn resolve_with(_: &F, _: usize, _: Self::Resolver, _: *mut Self::Archived) {}
}

impl<F, S: Fallible + ?Sized> SerializeWith<F, S> for FromContext {
  fn serialize_with(_: &F, _: &mut S) -> Result<(), S::Error> {
    Ok(())
  }
}

impl DeserializeWith<(), Arc<CompilerOptions>, CacheableDeserializer> for FromContext {
  fn deserialize_with(
    _: &(),
    d: &mut CacheableDeserializer,
  ) -> Result<Arc<CompilerOptions>, DeserializeError> {
    let ctx = unsafe { d.context::<CacheContext>() };
    Ok(ctx.options.clone())
  }
}

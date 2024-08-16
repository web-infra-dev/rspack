use rkyv::{
  bytecheck::BoolCheckError,
  collections::util::validation::ArchivedEntryError,
  out_field,
  ser::Serializer,
  string::{ArchivedString, StringResolver},
  validation::ArchiveContext,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Archive, CheckBytes, Deserialize, Fallible,
};
use rspack_resolver::AliasValue;

use super::AsPreset;

pub struct ArchivedAliasValue {
  is_ignore: bool,
  path: ArchivedString,
}

pub struct AliasValueResolver {
  path: StringResolver,
}

impl ArchiveWith<AliasValue> for AsPreset {
  type Archived = ArchivedAliasValue;
  type Resolver = AliasValueResolver;

  #[inline]
  unsafe fn resolve_with(
    field: &AliasValue,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    let (fp, fo) = out_field!(out.is_ignore);
    let is_ignore = matches!(field, AliasValue::Ignore);
    is_ignore.resolve(pos + fp, (), fo);

    let path = if let AliasValue::Path(path) = field {
      path
    } else {
      ""
    };
    let (fp, fo) = out_field!(out.path);
    ArchivedString::resolve_from_str(path, pos + fp, resolver.path, fo);
  }
}

impl<S: Fallible + Serializer + ?Sized> SerializeWith<AliasValue, S> for AsPreset {
  #[inline]
  fn serialize_with(field: &AliasValue, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    let path = if let AliasValue::Path(path) = field {
      path
    } else {
      ""
    };
    Ok(AliasValueResolver {
      path: ArchivedString::serialize_from_str(path, serializer)?,
    })
  }
}

impl<C> CheckBytes<C> for ArchivedAliasValue
where
  ArchivedString: CheckBytes<C>,
  C: ArchiveContext + ?Sized,
{
  type Error = ArchivedEntryError<BoolCheckError, <ArchivedString as CheckBytes<C>>::Error>;

  #[inline]
  unsafe fn check_bytes<'a>(value: *const Self, context: &mut C) -> Result<&'a Self, Self::Error> {
    bool::check_bytes(core::ptr::addr_of!((*value).is_ignore), context)
      .map_err(ArchivedEntryError::KeyCheckError)?;
    ArchivedString::check_bytes(core::ptr::addr_of!((*value).path), context)
      .map_err(ArchivedEntryError::ValueCheckError)?;
    Ok(&*value)
  }
}

impl<D> DeserializeWith<ArchivedAliasValue, AliasValue, D> for AsPreset
where
  D: ?Sized + Fallible,
{
  fn deserialize_with(
    field: &ArchivedAliasValue,
    deserializer: &mut D,
  ) -> Result<AliasValue, D::Error> {
    Ok(if field.is_ignore {
      AliasValue::Ignore
    } else {
      AliasValue::Path(field.path.deserialize(deserializer)?)
    })
  }
}

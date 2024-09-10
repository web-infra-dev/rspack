use rkyv::{
  bytecheck::{CheckBytes, StructCheckContext},
  rancor::{Fallible, Source, Trace},
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Archive, Deserialize, Place, Portable,
};
use rspack_resolver::AliasValue;

use super::AsPreset;

pub struct ArchivedAliasValue {
  is_ignore: bool,
  path: ArchivedString,
}

unsafe impl Portable for ArchivedAliasValue {}

pub struct AliasValueResolver {
  path: StringResolver,
}

impl ArchiveWith<AliasValue> for AsPreset {
  type Archived = ArchivedAliasValue;
  type Resolver = AliasValueResolver;

  #[inline]
  fn resolve_with(field: &AliasValue, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let field_ptr = unsafe { &raw mut (*out.ptr()).is_ignore };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    let is_ignore = matches!(field, AliasValue::Ignore);
    is_ignore.resolve((), field_out);
    let field_ptr = unsafe { &raw mut (*out.ptr()).path };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    let path = if let AliasValue::Path(path) = field {
      path
    } else {
      ""
    };
    ArchivedString::resolve_from_str(path, resolver.path, field_out);
  }
}

impl<S> SerializeWith<AliasValue, S> for AsPreset
where
  S: Fallible + Writer + ?Sized,
  S::Error: Source,
{
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

unsafe impl<C> CheckBytes<C> for ArchivedAliasValue
where
  ArchivedString: CheckBytes<C>,
  C: Fallible + ?Sized,
  C::Error: Trace,
  bool: CheckBytes<C>,
{
  unsafe fn check_bytes(value: *const Self, context: &mut C) -> Result<(), C::Error> {
    bool::check_bytes(core::ptr::addr_of!((*value).is_ignore), context).map_err(|e| {
      <C::Error as Trace>::trace(
        e,
        StructCheckContext {
          struct_name: "ArchivedAliasValue",
          field_name: "is_ignore",
        },
      )
    })?;
    ArchivedString::check_bytes(core::ptr::addr_of!((*value).path), context).map_err(|e| {
      <C::Error as Trace>::trace(
        e,
        StructCheckContext {
          struct_name: "ArchivedAliasValue",
          field_name: "path",
        },
      )
    })?;
    Ok(())
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

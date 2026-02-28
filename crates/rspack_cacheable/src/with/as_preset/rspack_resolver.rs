use rkyv::{
  Archive, Archived, Deserialize, Place, Portable, Resolver,
  bytecheck::{CheckBytes, StructCheckContext},
  de::Pooling,
  rancor::{Fallible, Trace},
  ser::{Sharing, Writer},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use rspack_resolver::AliasValue;

use super::AsPreset;
use crate::{ContextGuard, Error, utils::PortablePath};

pub struct ArchivedAliasValue {
  is_ignore: bool,
  path: Archived<PortablePath>,
}

unsafe impl Portable for ArchivedAliasValue {}

pub struct AliasValueResolver {
  inner: Resolver<PortablePath>,
  path: PortablePath,
}

impl ArchiveWith<AliasValue> for AsPreset {
  type Archived = ArchivedAliasValue;
  type Resolver = AliasValueResolver;

  #[inline]
  fn resolve_with(field: &AliasValue, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let AliasValueResolver { inner, path } = resolver;
    let field_ptr = unsafe { &raw mut (*out.ptr()).is_ignore };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    let is_ignore = matches!(field, AliasValue::Ignore);
    is_ignore.resolve((), field_out);
    let field_ptr = unsafe { &raw mut (*out.ptr()).path };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(&path, inner, field_out);
  }
}

impl<S> SerializeWith<AliasValue, S> for AsPreset
where
  S: Fallible<Error = Error> + Writer + Sharing<Error> + ?Sized,
{
  #[inline]
  fn serialize_with(field: &AliasValue, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    let guard = ContextGuard::sharing_guard(serializer)?;
    let path_str = if let AliasValue::Path(path) = field {
      path.as_str()
    } else {
      ""
    };
    let portable_path = PortablePath::new(path_str.as_ref(), guard.project_root());
    Ok(AliasValueResolver {
      inner: rkyv::Serialize::serialize(&portable_path, serializer)?,
      path: portable_path,
    })
  }
}

unsafe impl<C> CheckBytes<C> for ArchivedAliasValue
where
  Archived<PortablePath>: CheckBytes<C>,
  C: Fallible + ?Sized,
  C::Error: Trace,
  bool: CheckBytes<C>,
{
  unsafe fn check_bytes(value: *const Self, context: &mut C) -> Result<(), C::Error> {
    unsafe {
      bool::check_bytes(core::ptr::addr_of!((*value).is_ignore), context).map_err(|e| {
        <C::Error as Trace>::trace(
          e,
          StructCheckContext {
            struct_name: "ArchivedAliasValue",
            field_name: "is_ignore",
          },
        )
      })?;
    }
    unsafe {
      <Archived<PortablePath>>::check_bytes(core::ptr::addr_of!((*value).path), context).map_err(
        |e| {
          <C::Error as Trace>::trace(
            e,
            StructCheckContext {
              struct_name: "ArchivedAliasValue",
              field_name: "path",
            },
          )
        },
      )?;
    }
    Ok(())
  }
}

impl<D> DeserializeWith<ArchivedAliasValue, AliasValue, D> for AsPreset
where
  D: Fallible<Error = Error> + Pooling<Error> + ?Sized,
{
  fn deserialize_with(
    field: &ArchivedAliasValue,
    deserializer: &mut D,
  ) -> Result<AliasValue, D::Error> {
    Ok(if field.is_ignore {
      AliasValue::Ignore
    } else {
      let portable_path: PortablePath = Deserialize::deserialize(&field.path, deserializer)?;
      let guard = ContextGuard::pooling_guard(deserializer)?;
      AliasValue::Path(portable_path.into_path_string(guard.project_root()))
    })
  }
}

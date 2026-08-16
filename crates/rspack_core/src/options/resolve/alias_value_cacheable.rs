//! `rspack_cacheable` support for [`AliasValue`], living next to the only field that asks for it.
//!
//! It cannot live in `rspack_cacheable`: that would make the crate depend on the resolver, while
//! the resolver depends on `rspack_paths` which depends on `rspack_cacheable` — a cycle. Nor can
//! it reuse `AsPreset` here, since implementing a foreign trait for a foreign type is not allowed.
//! Hence the local [`AsAliasValue`] adapter.

use rspack_cacheable::{
  __private::rkyv::{
    Archive, Archived, Deserialize, Place, Portable, Resolver, Serialize,
    bytecheck::{CheckBytes, StructCheckContext},
    de::Pooling,
    rancor::{Fallible, Trace},
    ser::{Sharing, Writer},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
  },
  ContextGuard, Error,
  utils::PortablePath,
};
use rspack_resolver::AliasValue;

/// `with` adapter selecting this module's [`AliasValue`] serialization.
pub struct AsAliasValue;

pub struct ArchivedAliasValue {
  is_ignore: bool,
  path: Archived<PortablePath>,
}

unsafe impl Portable for ArchivedAliasValue {}

pub struct AliasValueResolver {
  inner: Resolver<PortablePath>,
  path: PortablePath,
}

impl ArchiveWith<AliasValue> for AsAliasValue {
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

impl<S> SerializeWith<AliasValue, S> for AsAliasValue
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
      inner: Serialize::serialize(&portable_path, serializer)?,
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

impl<D> DeserializeWith<ArchivedAliasValue, AliasValue, D> for AsAliasValue
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

#[cfg(test)]
mod tests {
  // Moved here from `rspack_cacheable_test` along with the implementation.
  use rspack_cacheable::{
    enable_cacheable as cacheable, from_bytes, to_bytes,
    with::{AsCacheable, AsOption, AsTuple2, AsVec},
  };
  use rspack_resolver::{Alias, AliasValue};

  use super::AsAliasValue;

  #[cacheable]
  #[derive(Debug, Clone, Hash, PartialEq, Eq)]
  struct ResolverOption {
    #[cacheable(with=AsOption<AsVec<AsTuple2<AsCacheable, AsVec<AsAliasValue>>>>)]
    alias: Option<Alias>,
  }

  #[test]
  fn test_preset_rspack_resolver() {
    let option = ResolverOption {
      alias: Some(vec![
        (
          String::from("@"),
          vec![AliasValue::Path(String::from("./src"))],
        ),
        (String::from("ignore"), vec![AliasValue::Ignore]),
        (
          String::from("components"),
          vec![
            AliasValue::Path(String::from("./components")),
            AliasValue::Path(String::from("./src")),
            AliasValue::Ignore,
          ],
        ),
      ]),
    };

    let bytes = to_bytes(&option, &()).unwrap();
    let new_option: ResolverOption = from_bytes(&bytes, &()).unwrap();
    assert_eq!(option, new_option);
  }
}

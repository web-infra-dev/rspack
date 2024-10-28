use rkyv::{
  bytecheck::{CheckBytes, StructCheckContext},
  rancor::{Fallible, Source, Trace},
  ser::{Allocator, Writer},
  string::{ArchivedString, StringResolver},
  vec::{ArchivedVec, VecResolver},
  Archive, Deserialize, Place, Portable, Serialize,
};

pub struct TypeWrapperRef<'a> {
  pub type_name: &'a str,
  pub bytes: &'a [u8],
}

pub struct ArchivedTypeWrapper {
  type_name: ArchivedString,
  bytes: ArchivedVec<u8>,
}

unsafe impl Portable for ArchivedTypeWrapper {}

pub struct TypeWrapper {
  pub type_name: String,
  pub bytes: Vec<u8>,
}

impl<'a> Archive for TypeWrapperRef<'a> {
  type Archived = ArchivedTypeWrapper;
  type Resolver = (StringResolver, VecResolver);

  #[inline]
  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let field_ptr = unsafe { &raw mut (*out.ptr()).type_name };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    ArchivedString::resolve_from_str(self.type_name, resolver.0, field_out);
    let field_ptr = unsafe { &raw mut (*out.ptr()).bytes };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    ArchivedVec::resolve_from_len(self.bytes.len(), resolver.1, field_out);
  }
}

impl Archive for TypeWrapper {
  type Archived = ArchivedTypeWrapper;
  type Resolver = (StringResolver, VecResolver);

  #[inline]
  fn resolve(&self, _resolver: Self::Resolver, _out: Place<Self::Archived>) {
    unreachable!()
  }
}

impl<'a, S> Serialize<S> for TypeWrapperRef<'a>
where
  S: ?Sized + Fallible + Writer + Allocator,
  S::Error: Source,
{
  #[inline]
  fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    Ok((
      ArchivedString::serialize_from_str(self.type_name, serializer)?,
      ArchivedVec::serialize_from_slice(self.bytes, serializer)?,
    ))
  }
}

unsafe impl<C> CheckBytes<C> for ArchivedTypeWrapper
where
  ArchivedString: CheckBytes<C>,
  ArchivedVec<u8>: CheckBytes<C>,
  C: Fallible + ?Sized,
  C::Error: Trace,
{
  unsafe fn check_bytes(bytes: *const Self, context: &mut C) -> Result<(), C::Error> {
    ArchivedString::check_bytes(core::ptr::addr_of!((*bytes).type_name), context).map_err(|e| {
      <C::Error as Trace>::trace(
        e,
        StructCheckContext {
          struct_name: "ArchivedTypeWrapper",
          field_name: "type_name",
        },
      )
    })?;
    ArchivedVec::<u8>::check_bytes(core::ptr::addr_of!((*bytes).bytes), context).map_err(|e| {
      <C::Error as Trace>::trace(
        e,
        StructCheckContext {
          struct_name: "ArchivedTypeWrapper",
          field_name: "bytes",
        },
      )
    })?;
    Ok(())
  }
}

impl<D> Deserialize<TypeWrapper, D> for ArchivedTypeWrapper
where
  D: Fallible + ?Sized,
  D::Error: Source,
{
  #[inline]
  fn deserialize(&self, deserializer: &mut D) -> Result<TypeWrapper, D::Error> {
    Ok(TypeWrapper {
      type_name: Deserialize::<String, D>::deserialize(&self.type_name, deserializer)?,
      bytes: Deserialize::<Vec<u8>, D>::deserialize(&self.bytes, deserializer)?,
    })
  }
}

use rkyv::{
  collections::util::validation::ArchivedEntryError,
  out_field,
  ser::{ScratchSpace, Serializer},
  string::{ArchivedString, StringResolver},
  validation::ArchiveContext,
  vec::{ArchivedVec, VecResolver},
  Archive, CheckBytes, Deserialize, Fallible, Serialize,
};

pub struct TypeWrapperRef<'a> {
  pub type_name: &'a str,
  pub bytes: &'a [u8],
}

pub struct ArchivedTypeWrapper {
  type_name: ArchivedString,
  bytes: ArchivedVec<u8>,
}

pub struct TypeWrapper {
  pub type_name: String,
  pub bytes: Vec<u8>,
}

impl<'a> Archive for TypeWrapperRef<'a> {
  type Archived = ArchivedTypeWrapper;
  type Resolver = (StringResolver, VecResolver);

  #[inline]
  unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
    let (fp, fo) = out_field!(out.type_name);
    ArchivedString::resolve_from_str(self.type_name, pos + fp, resolver.0, fo);

    let (fp, fo) = out_field!(out.bytes);
    ArchivedVec::resolve_from_len(self.bytes.len(), pos + fp, resolver.1, fo);
  }
}

impl Archive for TypeWrapper {
  type Archived = ArchivedTypeWrapper;
  type Resolver = (StringResolver, VecResolver);

  #[inline]
  unsafe fn resolve(&self, _pos: usize, _resolver: Self::Resolver, _out: *mut Self::Archived) {
    unreachable!()
  }
}

impl<'a, S> Serialize<S> for TypeWrapperRef<'a>
where
  S: Serializer + ScratchSpace + Fallible + ?Sized,
{
  #[inline]
  fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    Ok((
      ArchivedString::serialize_from_str(self.type_name, serializer)?,
      ArchivedVec::serialize_from_slice(self.bytes, serializer)?,
    ))
  }
}

impl<C> CheckBytes<C> for ArchivedTypeWrapper
where
  ArchivedString: CheckBytes<C>,
  ArchivedVec<u8>: CheckBytes<C>,
  C: ArchiveContext + ?Sized,
{
  type Error = ArchivedEntryError<
    <ArchivedString as CheckBytes<C>>::Error,
    <ArchivedVec<u8> as CheckBytes<C>>::Error,
  >;

  #[inline]
  unsafe fn check_bytes<'a>(bytes: *const Self, context: &mut C) -> Result<&'a Self, Self::Error> {
    ArchivedString::check_bytes(core::ptr::addr_of!((*bytes).type_name), context)
      .map_err(ArchivedEntryError::KeyCheckError)?;
    ArchivedVec::<u8>::check_bytes(core::ptr::addr_of!((*bytes).bytes), context)
      .map_err(ArchivedEntryError::ValueCheckError)?;
    Ok(&*bytes)
  }
}

impl<D> Deserialize<TypeWrapper, D> for ArchivedTypeWrapper
where
  D: Fallible + ?Sized,
{
  #[inline]
  fn deserialize(&self, deserializer: &mut D) -> Result<TypeWrapper, D::Error> {
    Ok(TypeWrapper {
      type_name: Deserialize::<String, D>::deserialize(&self.type_name, deserializer)?,
      bytes: Deserialize::<Vec<u8>, D>::deserialize(&self.bytes, deserializer)?,
    })
  }
}

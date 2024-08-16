use std::marker::PhantomData;

use rkyv::{
  out_field,
  ser::{ScratchSpace, Serializer},
  validation::ArchiveContext,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Archive, CheckBytes, Deserialize, Fallible, Serialize,
};

pub struct ArchivedWithWrapper<WA, W>(WA, PhantomData<W>);

impl<WA, W, C> CheckBytes<C> for ArchivedWithWrapper<WA, W>
where
  WA: CheckBytes<C>,
  C: ArchiveContext + ?Sized,
{
  type Error = <WA as CheckBytes<C>>::Error;

  #[inline]
  unsafe fn check_bytes<'a>(bytes: *const Self, context: &mut C) -> Result<&'a Self, Self::Error> {
    WA::check_bytes(core::ptr::addr_of!((*bytes).0), context)?;
    Ok(&*bytes)
  }
}

pub struct WithWrapper<A, W>(A, PhantomData<W>);

impl<A, W> WithWrapper<A, W> {
  pub fn into_inner(self) -> A {
    self.0
  }
}

impl<A, W> Archive for WithWrapper<A, W>
where
  W: ArchiveWith<A>,
{
  type Archived = ArchivedWithWrapper<W::Archived, W>;
  type Resolver = W::Resolver;

  #[inline]
  unsafe fn resolve(&self, _pos: usize, _resolver: Self::Resolver, _out: *mut Self::Archived) {
    unreachable!()
  }
}

pub struct WithWrapperRef<'a, A, W>(&'a A, PhantomData<W>);

impl<'a, A, W> WithWrapperRef<'a, A, W> {
  pub fn new(inner: &'a A) -> Self {
    Self(inner, Default::default())
  }
}

impl<'a, A, W> Archive for WithWrapperRef<'a, A, W>
where
  W: ArchiveWith<A>,
{
  type Archived = ArchivedWithWrapper<W::Archived, W>;
  type Resolver = W::Resolver;

  #[inline]
  unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
    let (fp, fo) = out_field!(out.0);
    W::resolve_with(self.0, pos + fp, resolver, fo)
  }
}

impl<'a, A, W, S> Serialize<S> for WithWrapperRef<'a, A, W>
where
  W: SerializeWith<A, S>,
  S: Serializer + ScratchSpace + Fallible + ?Sized,
{
  #[inline]
  fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    W::serialize_with(self.0, serializer)
  }
}

impl<A, W, D> Deserialize<WithWrapper<A, W>, D> for ArchivedWithWrapper<W::Archived, W>
where
  W: ArchiveWith<A> + DeserializeWith<W::Archived, A, D>,
  D: Fallible + ?Sized,
{
  #[inline]
  fn deserialize(&self, deserializer: &mut D) -> Result<WithWrapper<A, W>, D::Error> {
    Ok(WithWrapper(
      W::deserialize_with(&self.0, deserializer)?,
      PhantomData::default(),
    ))
  }
}

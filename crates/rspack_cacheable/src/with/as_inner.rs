use rkyv::{
  option::ArchivedOption,
  out_field,
  ser::{ScratchSpace, Serializer},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Fallible,
};

use crate::with::AsCacheable;

pub struct AsInner<T = AsCacheable> {
  _inner: T,
}

pub trait AsInnerConverter {
  type Inner;
  fn to_inner(&self) -> &Self::Inner;
  fn from_inner(data: Self::Inner) -> Self;
}

impl<T, O, A> ArchiveWith<T> for AsInner<A>
where
  T: AsInnerConverter<Inner = O>,
  A: ArchiveWith<O>,
{
  type Archived = A::Archived;
  type Resolver = A::Resolver;

  unsafe fn resolve_with(
    field: &T,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    A::resolve_with(field.to_inner(), pos, resolver, out)
  }
}

impl<T, A, O, S> SerializeWith<T, S> for AsInner<A>
where
  T: AsInnerConverter<Inner = O>,
  S: Fallible + ScratchSpace + Serializer + ?Sized,
  A: ArchiveWith<O> + SerializeWith<O, S>,
{
  fn serialize_with(field: &T, s: &mut S) -> Result<Self::Resolver, S::Error> {
    A::serialize_with(field.to_inner(), s)
  }
}

impl<T, A, O, D> DeserializeWith<A::Archived, T, D> for AsInner<A>
where
  T: AsInnerConverter<Inner = O>,
  A: ArchiveWith<O> + DeserializeWith<A::Archived, O, D>,
  D: ?Sized + Fallible,
{
  fn deserialize_with(field: &A::Archived, d: &mut D) -> Result<T, D::Error> {
    Ok(T::from_inner(A::deserialize_with(field, d)?))
  }
}

// for Arc
impl<T> AsInnerConverter for std::sync::Arc<T> {
  type Inner = T;
  fn to_inner(&self) -> &Self::Inner {
    self.as_ref()
  }
  fn from_inner(data: Self::Inner) -> Self {
    Self::new(data)
  }
}

// for OnceCell
// rkyv::with::Map
#[repr(u8)]
enum ArchivedOptionTag {
  None,
  Some,
}
#[repr(C)]
struct ArchivedOptionVariantNone(ArchivedOptionTag);
#[repr(C)]
struct ArchivedOptionVariantSome<T>(ArchivedOptionTag, T);
impl<O, A> ArchiveWith<once_cell::sync::OnceCell<O>> for AsInner<A>
where
  A: ArchiveWith<O>,
{
  type Archived = ArchivedOption<<A as ArchiveWith<O>>::Archived>;
  type Resolver = Option<<A as ArchiveWith<O>>::Resolver>;

  unsafe fn resolve_with(
    field: &once_cell::sync::OnceCell<O>,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    match resolver {
      None => {
        let out = out.cast::<ArchivedOptionVariantNone>();
        core::ptr::addr_of_mut!((*out).0).write(ArchivedOptionTag::None);
      }
      Some(resolver) => {
        let out = out.cast::<ArchivedOptionVariantSome<<A as ArchiveWith<O>>::Archived>>();
        core::ptr::addr_of_mut!((*out).0).write(ArchivedOptionTag::Some);

        let value = if let Some(value) = field.get() {
          value
        } else {
          core::hint::unreachable_unchecked();
        };

        let (fp, fo) = out_field!(out.1);
        A::resolve_with(value, pos + fp, resolver, fo);
      }
    }
  }
}

impl<A, O, S> SerializeWith<once_cell::sync::OnceCell<O>, S> for AsInner<A>
where
  S: Fallible + ScratchSpace + Serializer + ?Sized,
  A: ArchiveWith<O> + SerializeWith<O, S>,
{
  fn serialize_with(
    field: &once_cell::sync::OnceCell<O>,
    s: &mut S,
  ) -> Result<Self::Resolver, S::Error> {
    Ok(match field.get() {
      Some(inner) => Some(A::serialize_with(inner, s)?),
      None => None,
    })
  }
}

impl<A, O, D>
  DeserializeWith<ArchivedOption<<A as ArchiveWith<O>>::Archived>, once_cell::sync::OnceCell<O>, D>
  for AsInner<A>
where
  D: Fallible + ?Sized,
  A: ArchiveWith<O> + DeserializeWith<<A as ArchiveWith<O>>::Archived, O, D>,
{
  fn deserialize_with(
    field: &ArchivedOption<<A as ArchiveWith<O>>::Archived>,
    d: &mut D,
  ) -> Result<once_cell::sync::OnceCell<O>, D::Error> {
    match field {
      ArchivedOption::Some(value) => Ok(once_cell::sync::OnceCell::with_value(
        A::deserialize_with(value, d)?,
      )),
      ArchivedOption::None => Ok(once_cell::sync::OnceCell::new()),
    }
  }
}

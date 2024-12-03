use rkyv::{
  munge::munge,
  option::ArchivedOption,
  rancor::Fallible,
  traits::NoUndef,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
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

  #[inline]
  fn resolve_with(field: &T, resolver: Self::Resolver, out: Place<Self::Archived>) {
    A::resolve_with(field.to_inner(), resolver, out)
  }
}

impl<T, A, O, S> SerializeWith<T, S> for AsInner<A>
where
  T: AsInnerConverter<Inner = O>,
  S: Fallible + ?Sized,
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
  D: Fallible + ?Sized,
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

// SAFETY: `ArchivedOptionTag` is `repr(u8)` and so always consists of a single
// well-defined byte.
unsafe impl NoUndef for ArchivedOptionTag {}

impl<O, A> ArchiveWith<once_cell::sync::OnceCell<O>> for AsInner<A>
where
  A: ArchiveWith<O>,
{
  type Archived = ArchivedOption<<A as ArchiveWith<O>>::Archived>;
  type Resolver = Option<<A as ArchiveWith<O>>::Resolver>;

  fn resolve_with(
    field: &once_cell::sync::OnceCell<O>,
    resolver: Self::Resolver,
    out: Place<Self::Archived>,
  ) {
    // port rkyv::with::Map
    match resolver {
      None => {
        let out = unsafe { out.cast_unchecked::<ArchivedOptionVariantNone>() };
        munge!(let ArchivedOptionVariantNone(tag) = out);
        tag.write(ArchivedOptionTag::None);
      }
      Some(resolver) => {
        let out = unsafe {
          out.cast_unchecked::<ArchivedOptionVariantSome<<A as ArchiveWith<O>>::Archived>>()
        };
        munge!(let ArchivedOptionVariantSome(tag, out_value) = out);
        tag.write(ArchivedOptionTag::Some);

        let value = if let Some(value) = field.get() {
          value
        } else {
          unsafe {
            core::hint::unreachable_unchecked();
          }
        };

        A::resolve_with(value, resolver, out_value);
      }
    }
  }
}

impl<A, O, S> SerializeWith<once_cell::sync::OnceCell<O>, S> for AsInner<A>
where
  S: Fallible + ?Sized,
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

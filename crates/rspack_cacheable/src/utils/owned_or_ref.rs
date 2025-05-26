/// A owned-or-ref enum
#[derive(Debug, Eq)]
pub enum OwnedOrRef<'a, T> {
  Borrowed(&'a T),
  Owned(T),
}

impl<'a, 'b, A, B> PartialEq<OwnedOrRef<'a, A>> for OwnedOrRef<'b, B>
where
  B: PartialEq<A>,
{
  #[inline]
  fn eq(&self, other: &OwnedOrRef<'a, A>) -> bool {
    PartialEq::eq(self.as_ref(), other.as_ref())
  }
}

impl<T> From<T> for OwnedOrRef<'_, T> {
  fn from(value: T) -> Self {
    Self::Owned(value)
  }
}

impl<'a, T> From<&'a T> for OwnedOrRef<'a, T> {
  fn from(value: &'a T) -> Self {
    Self::Borrowed(value)
  }
}

impl<T> AsRef<T> for OwnedOrRef<'_, T> {
  fn as_ref(&self) -> &T {
    match self {
      Self::Borrowed(b) => b,
      Self::Owned(o) => o,
    }
  }
}

impl<T> OwnedOrRef<'_, T> {
  pub fn into_owned(self) -> T {
    match self {
      Self::Borrowed(_) => panic!("Borrowed data does not allow call into_owned"),
      Self::Owned(o) => o,
    }
  }
}

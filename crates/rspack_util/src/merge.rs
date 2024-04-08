pub trait MergeFrom: Clone {
  fn merge_from(self, other: &Self) -> Self;
}

macro_rules! impl_merge_from {
  ($t:ty) => {
    impl MergeFrom for $t {
      fn merge_from(self, other: &Self) -> Self {
        other.clone()
      }
    }
  };
  ($($t:ty),+) => {
    $(impl_merge_from!($t);)+
  };
}

impl<T: MergeFrom> MergeFrom for Option<T> {
  fn merge_from(self, other: &Self) -> Self {
    merge_from_optional_with(self, other.as_ref(), |a, b| a.merge_from(b))
  }
}

impl_merge_from!(i8, i16, i32, i64, i128);
impl_merge_from!(u8, u16, u32, u64, u128);
impl_merge_from!(bool);
impl_merge_from!(String);

pub fn merge_from_optional_with<T: MergeFrom>(
  base: Option<T>,
  other: Option<&T>,
  f: impl FnOnce(T, &T) -> T,
) -> Option<T> {
  match (base, other) {
    (None, None) => None,
    (None, Some(_)) => other.cloned(),
    (Some(base), None) => Some(base),
    (Some(base), Some(other)) => Some(f(base, other)),
  }
}

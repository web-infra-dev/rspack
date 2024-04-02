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
    match (&self, other) {
      (None, None) => None,
      (Some(_), None) => self,
      (None, Some(_)) => other.clone(),
      (Some(a), Some(b)) => Some(a.clone().merge_from(b)),
    }
  }
}

impl_merge_from!(i8, i16, i32, i64, i128);
impl_merge_from!(u8, u16, u32, u64, u128);
impl_merge_from!(bool);
impl_merge_from!(String);

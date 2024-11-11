use rspack_regex::RspackRegex;

use crate::atom::Atom;

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

impl MergeFrom for Vec<String> {
  fn merge_from(self, other: &Self) -> Self {
    let mut res = Vec::new();
    for item in other {
      if item == "..." {
        res.extend(self.clone());
      } else {
        res.push(item.clone());
      }
    }
    res
  }
}

impl_merge_from!(i8, i16, i32, i64, i128);
impl_merge_from!(u8, u16, u32, u64, u128);
impl_merge_from!(f64);
impl_merge_from!(bool);
impl_merge_from!(String);
impl_merge_from!(Atom);
impl_merge_from!(RspackRegex);

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

use std::fmt::Debug;

use napi::{NapiRaw, NapiValue};

/// The concept is similar to `Either`, but `Either` is a common used name. To avoid conflictcations I
/// borrow the `Union` name from the `union type` of TypeScript.
pub enum Union<A, B> {
  Left(A),
  Right(B),
}

impl<Left, Right> Union<Left, Right> {
  pub fn expect_left(self) -> Left {
    match self {
      Union::Left(l) => l,
      Union::Right(_) => panic!("expect left, but got right"),
    }
  }
  pub fn expect_right(self) -> Right {
    match self {
      Union::Left(_) => panic!("expect right, but got left"),
      Union::Right(r) => r,
    }
  }
}

// TODO(hyf0): this is really not ideal. I will fix it when I understand
// https://users.rust-lang.org/t/check-if-a-trait-is-implemented-or-not/73756
impl<A, B> Debug for Union<A, B> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Left(_) => f.debug_tuple("Left").field(&"?Debug").finish(),
      Self::Right(_) => f.debug_tuple("Right").field(&"?Debug").finish(),
    }
  }
}

impl<A: NapiRaw + NapiValue, B: NapiRaw + NapiValue> NapiRaw for Union<A, B> {
  unsafe fn raw(&self) -> napi::sys::napi_value {
    match self {
      Union::Left(a) => a.raw(),
      Union::Right(b) => b.raw(),
    }
  }
}

impl<A: NapiRaw + NapiValue, B: NapiRaw + NapiValue> NapiValue for Union<A, B> {
  unsafe fn from_raw(env: napi::sys::napi_env, value: napi::sys::napi_value) -> napi::Result<Self> {
    let a = A::from_raw(env, value);
    if a.is_ok() {
      return a.map(|a| Union::Left(a));
    }
    B::from_raw(env, value).map(|b| Union::Right(b))
  }

  unsafe fn from_raw_unchecked(env: napi::sys::napi_env, value: napi::sys::napi_value) -> Self {
    Self::from_raw(env, value).expect("Should not failed")
  }
}

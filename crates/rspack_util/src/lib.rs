#![feature(int_roundings)]

mod merge;

pub mod comparators;
pub mod ext;
pub mod fx_dashmap;
pub mod identifier;
pub mod infallible;
pub mod number_hash;
pub mod path;
pub mod source_map;
pub mod swc;

use std::future::Future;

pub use merge::{merge_from_optional_with, MergeFrom};

pub async fn try_any<T, Fut, F, E>(it: impl IntoIterator<Item = T>, f: F) -> Result<bool, E>
where
  Fut: Future<Output = Result<bool, E>>,
  F: Fn(T) -> Fut,
{
  let it = it.into_iter();
  for i in it {
    if f(i).await? {
      return Ok(true);
    }
  }
  Ok(false)
}

pub fn try_any_sync<T, F, E>(it: impl IntoIterator<Item = T>, f: F) -> Result<bool, E>
where
  F: Fn(T) -> Result<bool, E>,
{
  let it = it.into_iter();
  for i in it {
    if f(i)? {
      return Ok(true);
    }
  }
  Ok(false)
}

pub async fn try_all<T, Fut, F, E>(it: impl IntoIterator<Item = T>, f: F) -> Result<bool, E>
where
  Fut: Future<Output = Result<bool, E>>,
  F: Fn(T) -> Fut,
{
  let it = it.into_iter();
  for i in it {
    if !(f(i).await?) {
      return Ok(false);
    }
  }
  Ok(true)
}

#![feature(const_type_name)]

mod ukey;
pub use ukey::*;
mod database;
pub use database::*;

pub trait DatabaseItem
where
  Self: Sized,
{
  fn ukey(&self) -> Ukey<Self>;
}

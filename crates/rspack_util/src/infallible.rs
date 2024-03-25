use std::convert::Infallible;

pub trait ResultInfallibleExt {
  type Ok;
  fn always_ok(self) -> Self::Ok;
}

impl<T> ResultInfallibleExt for Result<T, Infallible> {
  type Ok = T;
  fn always_ok(self) -> T {
    match self {
      Ok(ok) => ok,
      Err(infallible) => match infallible {},
    }
  }
}

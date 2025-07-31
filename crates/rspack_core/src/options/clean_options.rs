use futures::future::BoxFuture;
use rspack_error::Result;

pub type KeepFn = Box<dyn for<'a> Fn(&'a str) -> BoxFuture<'a, Result<bool>> + Sync + Send>;

#[derive(Default)]
pub struct CleanOptions {
  pub dry: bool,
  pub keep: Option<KeepFn>,
}

impl std::fmt::Debug for CleanOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CleanOptions")
      .field("dry", &self.dry)
      .field("keep", &self.keep.as_ref().map(|_| "KeepFn"))
      .finish()
  }
}

impl Clone for CleanOptions {
  fn clone(&self) -> Self {
    Self {
      dry: self.dry,
      keep: None, // KeepFn cannot be cloned
    }
  }
}

use napi_derive::napi;

#[napi]
pub struct Compilation(pub(crate) crate::Compilation);

#[napi]
impl Compilation {
  #[napi(getter)]
  pub fn hash(&self) -> napi::Result<Option<&str>> {
    Ok(self.0.get_hash())
  }
}

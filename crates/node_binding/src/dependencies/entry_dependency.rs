use napi::{Either, Env, JsString};

#[napi]
pub struct EntryDependency {
  pub(crate) request: String,
  pub(crate) dependency_id: Option<rspack_core::DependencyId>,
}

impl EntryDependency {
  pub fn resolve(
    &mut self,
    context: rspack_core::Context,
    layer: Option<String>,
  ) -> napi::Result<Box<dyn rspack_core::Dependency>> {
    match &self.dependency_id {
        Some(dependency_id) => {
          Err(napi::Error::from_reason(format!(
            "Dependency with id = {:?} has already been resolved. Reusing EntryDependency is not allowed because Rust requires its ownership.",
            dependency_id
        )))
        }
        None => {
          let dependency = Box::new(rspack_core::EntryDependency::new(
            self.request.to_string(),
            context,
            layer,
            false,
          )) as rspack_core::BoxDependency;
          self.dependency_id = Some(*dependency.id());
          Ok(dependency)
        }
      }
  }
}

#[napi]
impl EntryDependency {
  #[napi(constructor)]
  pub fn new(request: String) -> Self {
    Self {
      request,
      dependency_id: None,
    }
  }

  #[napi(getter)]
  pub fn get_type(&mut self) -> napi::Result<&str> {
    Ok("entry")
  }

  #[napi(getter)]
  pub fn category(&mut self) -> napi::Result<&str> {
    Ok("esm")
  }

  #[napi(getter)]
  pub fn request(&mut self, env: Env) -> napi::Result<napi::Either<JsString, ()>> {
    Ok(Either::A(env.create_string(&self.request)?))
  }
}

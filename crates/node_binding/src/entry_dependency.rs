use napi::{Either, Env, JsString};
use napi_derive::napi;
use rspack_core::{BoxDependency, CompilerId, Context, Dependency, EntryDependency};

use crate::JsDependency;
use crate::JsDependencyBinding;
use crate::COMPILER_REFERENCES;

#[napi]
#[repr(C)]
pub struct JsEntryDependency {
  pub(crate) dependency: JsDependency,
  pub(crate) request: String,
}

impl JsEntryDependency {
  pub fn resolve(
    &mut self,
    compiler_id: CompilerId,
    context: Context,
    layer: Option<String>,
  ) -> napi::Result<Box<dyn Dependency>> {
    match &mut self.dependency.0 {
      Some(binding) => {
        Err(napi::Error::from_reason(format!(
          "Dependency with id = {:?} has already been resolved. Reusing JsEntryDependency is not allowed because Rust requires its ownership.",
          binding.dependency_id
      )))
      }
      None => {
        let dependency = Box::new(EntryDependency::new(
          self.request.to_string(),
          context,
          layer,
          false,
        )) as BoxDependency;
        let dependency_id = *dependency.id();

        // JsEntryDependency only relies on JsDependency for method access and does not create an instance in JavaScript.
        // Therefore, we do not use JsDependencyWrapper here.
        match COMPILER_REFERENCES.with(|ref_cell| {
          let references = ref_cell.borrow();
          references.get(&compiler_id).cloned()
        }) {
          Some(compiler_reference) => {
            self.dependency = JsDependency(Some(JsDependencyBinding{
              dependency_id,
              dependency: None,
              compiler_id,
              compiler_reference,
            }));
          }
          None => {
            return Err(napi::Error::from_reason(format!(
              "Unable to construct dependency with id = {:?} now. The Compiler has been garbage collected by JavaScript.",
              dependency_id
            )));
          }
        }

        Ok(dependency)
      }
    }
  }
}

#[napi]
impl JsEntryDependency {
  #[napi(constructor)]
  pub fn new(request: String) -> napi::Result<Self> {
    Ok(Self {
      request,
      dependency: JsDependency(None),
    })
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

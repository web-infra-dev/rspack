use napi::{Either, Env, JsString};
use napi_derive::napi;
use rspack_core::{BoxDependency, CompilerId, Context, Dependency, EntryDependency};

use crate::JsDependency;
use crate::COMPILER_REFERENCES;

#[napi]
pub struct JsEntryDependency {
  pub(crate) request: String,
  pub(crate) parent: Option<JsDependency>,
}

impl JsEntryDependency {
  pub fn resolve(
    &mut self,
    compiler_id: CompilerId,
    context: Context,
    layer: Option<String>,
  ) -> napi::Result<Box<dyn Dependency>> {
    match &mut self.parent {
      Some(dependency) => {
        Err(napi::Error::from_reason(format!(
          "Dependency with id = {:?} has already been resolved. Reusing JsEntryDependency is not allowed because Rust requires its ownership.",
          dependency.dependency_id
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
        println!("JsEntryDependency::resolve: dependency_id = {:?}", dependency_id);

        // JsEntryDependency only relies on JsDependency for method access and does not create an instance in JavaScript.
        // Therefore, we do not use JsDependencyWrapper here.
        match COMPILER_REFERENCES.with(|ref_cell| {
          let references = ref_cell.borrow();
          references.get(&compiler_id).cloned()
        }) {
          Some(compiler_reference) => {
            self.parent = Some(JsDependency {
              dependency_id,
              dependency: None,
              compiler_id,
              compiler_reference,
            });
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
      parent: None,
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

  #[napi(getter)]
  pub fn critical(&mut self) -> napi::Result<bool> {
    match &mut self.parent {
      Some(dep) => dep.critical(),
      None => Ok(false),
    }
  }

  #[napi(setter)]
  pub fn set_critical(&mut self, val: bool) -> napi::Result<()> {
    match &mut self.parent {
      Some(dep) => dep.set_critical(val),
      None => Ok(()),
    }
  }

  #[napi(getter)]
  pub fn ids(&mut self, env: Env) -> napi::Result<Either<Vec<JsString>, ()>> {
    match &mut self.parent {
      Some(dep) => dep.ids(env),
      None => Ok(Either::B(())),
    }
  }
}

use napi::{
  bindgen_prelude::{FromNapiValue, Function, Object, Result},
  Either, Env, JsObject, JsString, JsUnknown,
};
use napi_derive::napi;
use rspack_core::{BoxDependency, CompilerId, Context, Dependency, EntryDependency};

use crate::JsDependency;
use crate::COMPILER_REFERENCES;

#[napi(js_name = "EntryDependency")]
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

fn get_class_constructor(env: Env, name: &'static str) -> Result<Object> {
  #[allow(clippy::unwrap_used)]
  let ctor_ref = napi::bindgen_prelude::get_class_constructor(name).unwrap();
  let mut ctor_napi_val = std::ptr::null_mut();
  unsafe {
    napi::check_status!(
      napi::sys::napi_get_reference_value(env.raw(), ctor_ref, &mut ctor_napi_val),
      "Failed to get constructor reference of class `{}`",
      name
    )?
  };
  unsafe { Object::from_napi_value(env.raw(), ctor_napi_val) }
}

#[module_exports]
fn init(_exports: JsObject, env: Env) -> Result<()> {
  let global = env.get_global()?;
  let global_object = global
    .get_named_property::<JsUnknown>("Object")?
    .coerce_to_object()?;
  let set_prototype_of = global_object
    .get_named_property::<Function<(&Object, &Object), JsUnknown>>("setPrototypeOf")?;

  let css_module_prototype =
    get_class_constructor(env, "CssModule\0")?.get_named_property::<Object>("prototype")?;
  let normal_module_prototype =
    get_class_constructor(env, "NormalModule\0")?.get_named_property::<Object>("prototype")?;

  set_prototype_of.apply(
    global_object,
    (&css_module_prototype, &normal_module_prototype),
  )?;

  Ok(())
}

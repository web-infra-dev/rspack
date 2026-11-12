use napi::{
  Env, JsValue,
  bindgen_prelude::{Array, FromNapiValue, Unknown},
};
use rspack_core::CompilationId;

use super::with_module_graph;
use crate::module::{ModuleObject, ModuleObjectRef};

/// Read-only operations on a compilation's module collection.
#[napi]
pub struct Modules {
  compilation_id: CompilationId,
}

impl Modules {
  pub fn new(compilation_id: CompilationId) -> Self {
    Self { compilation_id }
  }
}

#[napi]
impl Modules {
  #[napi]
  pub fn size(&self) -> napi::Result<u32> {
    with_module_graph(self.compilation_id, |compilation| {
      Ok(compilation.get_module_graph().modules_len() as u32)
    })
  }

  #[napi(ts_args_type = "value: Module")]
  pub fn has(&self, env: &Env, value: Unknown<'_>) -> napi::Result<bool> {
    // SAFETY: Convert the live value with the matching N-API environment,
    // before borrowing the compilation's module graph.
    let module = unsafe { ModuleObjectRef::from_napi_value(env.raw(), value.raw()) };
    with_module_graph(self.compilation_id, |compilation| {
      // Non-module values are absent from the collection, like Set.has().
      let Ok(module) = module else {
        return Ok(false);
      };
      Ok(
        module.compiler_id == compilation.compiler_id()
          && compilation
            .module_by_identifier(&module.identifier)
            .is_some(),
      )
    })
  }

  #[napi(ts_return_type = "ReadonlyArray<Module>")]
  pub fn values<'env>(&self, env: &'env Env) -> napi::Result<Array<'env>> {
    with_module_graph(self.compilation_id, |compilation| {
      let module_graph = compilation.get_module_graph();
      let mut array = env.create_array(module_graph.modules_len() as u32)?;
      for (i, identifier) in module_graph.modules_keys().enumerate() {
        array.set(
          i as u32,
          compilation
            .module_by_identifier(identifier)
            .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id())),
        )?;
      }
      Ok(array)
    })
  }
}

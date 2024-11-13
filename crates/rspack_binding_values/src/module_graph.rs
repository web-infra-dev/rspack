use std::{ptr::NonNull, sync::Arc};

use napi::Either;
use napi_derive::napi;
use rspack_core::{Compilation, ModuleGraph, RuntimeSpec};
use rustc_hash::FxHashSet;

use crate::{JsDependency, JsModule, JsModuleWrapper};

#[napi]
pub struct JsModuleGraph {
  compilation: NonNull<Compilation>,
}

impl JsModuleGraph {
  fn as_ref(&self) -> napi::Result<(&'static Compilation, ModuleGraph<'static>)> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();

    Ok((compilation, module_graph))
  }
}

#[napi]
impl JsModuleGraph {
  #[napi(ts_return_type = "JsModule | null")]
  pub fn get_module(&self, js_dependency: &JsDependency) -> napi::Result<Option<JsModuleWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;
    let module = module_graph.get_module_by_dependency_id(&js_dependency.id());
    let js_module = module
      .map(|module| JsModuleWrapper::new(module.as_ref(), compilation.id(), Some(&compilation)));
    Ok(js_module)
  }

  #[napi]
  pub fn get_used_exports(
    &self,
    js_module: &JsModule,
    js_runtime: Either<String, Vec<String>>,
  ) -> napi::Result<Option<Either<bool, Vec<String>>>> {
    let (_, module_graph) = self.as_ref()?;

    let mut runtime: FxHashSet<Arc<str>> = FxHashSet::default();
    match js_runtime {
      Either::A(s) => {
        runtime.insert(Arc::from(s));
      }
      Either::B(vec) => {
        runtime.extend(vec.into_iter().map(Arc::from));
      }
    };
    let used_exports =
      module_graph.get_used_exports(&js_module.identifier, Some(&RuntimeSpec::new(runtime)));
    Ok(match used_exports {
      rspack_core::UsedExports::Null => None,
      rspack_core::UsedExports::Bool(b) => Some(Either::A(b)),
      rspack_core::UsedExports::Vec(vec) => Some(Either::B(
        vec
          .into_iter()
          .map(|atom| atom.to_string())
          .collect::<Vec<_>>(),
      )),
    })
  }
}

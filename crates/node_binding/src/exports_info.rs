use std::{ptr::NonNull, sync::Arc};

use napi::Either;
use napi_derive::napi;
use rspack_core::{Compilation, ExportsInfo, ModuleGraph, RuntimeSpec, UsedName};

use crate::JsRuntimeSpec;

#[napi]
pub struct JsExportsInfo {
  exports_info: ExportsInfo,
  compilation: NonNull<Compilation>,
}

impl JsExportsInfo {
  pub fn new(exports_info: ExportsInfo, compilation: &Compilation) -> Self {
    #[allow(clippy::unwrap_used)]
    Self {
      exports_info,
      compilation: NonNull::new(compilation as *const Compilation as *mut Compilation).unwrap(),
    }
  }

  fn as_ref(&self) -> napi::Result<ModuleGraph<'static>> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();
    Ok(module_graph)
  }

  fn as_mut(&mut self) -> napi::Result<ModuleGraph<'static>> {
    let compilation = unsafe { self.compilation.as_mut() };
    let module_graph = compilation.get_module_graph_mut();
    Ok(module_graph)
  }
}

#[napi]
impl JsExportsInfo {
  #[napi(ts_args_type = "runtime: string | string[] | undefined")]
  pub fn is_used(&self, js_runtime: JsRuntimeSpec) -> napi::Result<bool> {
    let module_graph = self.as_ref()?;
    let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
      Either::A(str) => std::iter::once(str).map(Arc::from).collect(),
      Either::B(vec) => vec.into_iter().map(Arc::from).collect(),
    });
    Ok(self.exports_info.is_used(&module_graph, runtime.as_ref()))
  }

  #[napi(ts_args_type = "runtime: string | string[] | undefined")]
  pub fn is_module_used(&self, js_runtime: JsRuntimeSpec) -> napi::Result<bool> {
    let module_graph = self.as_ref()?;
    let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
      Either::A(str) => std::iter::once(str).map(Arc::from).collect(),
      Either::B(vec) => vec.into_iter().map(Arc::from).collect(),
    });
    Ok(
      self
        .exports_info
        .is_module_used(&module_graph, runtime.as_ref()),
    )
  }

  #[napi(ts_args_type = "runtime: string | string[] | undefined")]
  pub fn set_used_in_unknown_way(&mut self, js_runtime: JsRuntimeSpec) -> napi::Result<bool> {
    let mut module_graph = self.as_mut()?;
    let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
      Either::A(str) => std::iter::once(str).map(Arc::from).collect(),
      Either::B(vec) => vec.into_iter().map(Arc::from).collect(),
    });
    Ok(
      self
        .exports_info
        .set_used_in_unknown_way(&mut module_graph, runtime.as_ref()),
    )
  }

  #[napi(
    ts_args_type = "name: string | string[], runtime: string | string[] | undefined",
    ts_return_type = " 0 | 1 | 2 | 3 | 4"
  )]
  pub fn get_used(
    &self,
    js_name: Either<String, Vec<String>>,
    js_runtime: JsRuntimeSpec,
  ) -> napi::Result<u32> {
    let module_graph = self.as_ref()?;
    let name = match js_name {
      Either::A(s) => UsedName::Str(s.into()),
      Either::B(v) => UsedName::Vec(v.into_iter().map(Into::into).collect::<Vec<_>>()),
    };
    let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
      Either::A(str) => std::iter::once(str).map(Arc::from).collect(),
      Either::B(vec) => vec.into_iter().map(Arc::from).collect(),
    });
    Ok(
      self
        .exports_info
        .get_used(&module_graph, name, runtime.as_ref()) as u32,
    )
  }
}

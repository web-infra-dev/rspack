use std::ptr::NonNull;

use napi::Either;
use napi_derive::napi;
use rspack_core::{
  Compilation, ExportsInfo, ExportsInfoGetter, ModuleGraphMut, ModuleGraphRef,
  PrefetchExportsInfoMode, RuntimeSpec,
};
use rspack_util::atom::Atom;

use crate::runtime::JsRuntimeSpec;

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

  fn as_ref(&self) -> napi::Result<ModuleGraphRef<'static>> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();
    Ok(module_graph)
  }

  fn as_mut(&mut self) -> napi::Result<ModuleGraphMut<'static>> {
    let compilation = unsafe { self.compilation.as_mut() };
    let module_graph =
      Compilation::get_make_module_graph_mut(&mut compilation.build_module_graph_artifact);
    Ok(module_graph)
  }
}

#[napi]
impl JsExportsInfo {
  #[napi(ts_args_type = "runtime: string | string[] | undefined")]
  pub fn is_used(&self, js_runtime: JsRuntimeSpec) -> napi::Result<bool> {
    let module_graph = self.as_ref()?;
    let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
      Either::A(str) => std::iter::once(str).map(Into::into).collect(),
      Either::B(vec) => vec.into_iter().map(Into::into).collect(),
    });
    let exports_info = ExportsInfoGetter::prefetch_used_info_without_name(
      &self.exports_info,
      &module_graph,
      runtime.as_ref(),
      false,
    );
    Ok(exports_info.is_used())
  }

  #[napi(ts_args_type = "runtime: string | string[] | undefined")]
  pub fn is_module_used(&self, js_runtime: JsRuntimeSpec) -> napi::Result<bool> {
    let module_graph = self.as_ref()?;
    let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
      Either::A(str) => std::iter::once(str).map(Into::into).collect(),
      Either::B(vec) => vec.into_iter().map(Into::into).collect(),
    });
    let exports_info = ExportsInfoGetter::prefetch_used_info_without_name(
      &self.exports_info,
      &module_graph,
      runtime.as_ref(),
      false,
    );
    Ok(exports_info.is_module_used())
  }

  #[napi(ts_args_type = "runtime: string | string[] | undefined")]
  pub fn set_used_in_unknown_way(&mut self, js_runtime: JsRuntimeSpec) -> napi::Result<bool> {
    let mut module_graph = self.as_mut()?;
    let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
      Either::A(str) => std::iter::once(str).map(Into::into).collect(),
      Either::B(vec) => vec.into_iter().map(Into::into).collect(),
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
    let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
      Either::A(str) => std::iter::once(str).map(Into::into).collect(),
      Either::B(vec) => vec.into_iter().map(Into::into).collect(),
    });
    let names = match js_name {
      Either::A(s) => vec![Atom::from(s)],
      Either::B(v) => v.into_iter().map(Into::into).collect::<Vec<_>>(),
    };
    let exports_info = ExportsInfoGetter::prefetch(
      &self.exports_info,
      &module_graph,
      PrefetchExportsInfoMode::Nested(&names),
    );
    let used = exports_info.get_used(&names, runtime.as_ref());
    Ok(used as u32)
  }
}

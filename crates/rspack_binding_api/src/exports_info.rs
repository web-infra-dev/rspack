use napi::Either;
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId, ExportsInfo, ModuleGraph, RuntimeSpec};
use rspack_util::atom::Atom;

use crate::{runtime::JsRuntimeSpec, with_compilation, with_compilation_mut};

#[napi]
pub struct JsExportsInfo {
  exports_info: ExportsInfo,
  compilation_id: CompilationId,
}

impl JsExportsInfo {
  pub fn new(exports_info: ExportsInfo, compilation: &Compilation) -> Self {
    Self {
      exports_info,
      compilation_id: compilation.id(),
    }
  }

  fn with_compilation<R>(
    &self,
    f: impl FnOnce(&Compilation) -> napi::Result<R>,
  ) -> napi::Result<R> {
    with_compilation(self.compilation_id, f)
  }

  fn with_compilation_mut<R>(
    &mut self,
    f: impl FnOnce(&mut Compilation) -> napi::Result<R>,
  ) -> napi::Result<R> {
    with_compilation_mut(self.compilation_id, f)
  }
}

#[napi]
impl JsExportsInfo {
  #[napi(ts_args_type = "runtime: string | string[] | undefined")]
  pub fn is_used(&self, js_runtime: JsRuntimeSpec) -> napi::Result<bool> {
    let exports_info = self.exports_info;
    self.with_compilation(|compilation| {
      let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
        Either::A(str) => std::iter::once(str).map(Into::into).collect(),
        Either::B(vec) => vec.into_iter().map(Into::into).collect(),
      });
      Ok(
        exports_info
          .as_data(&compilation.exports_info_artifact)
          .is_used(runtime.as_ref()),
      )
    })
  }

  #[napi(ts_args_type = "runtime: string | string[] | undefined")]
  pub fn is_module_used(&self, js_runtime: JsRuntimeSpec) -> napi::Result<bool> {
    let exports_info = self.exports_info;
    self.with_compilation(|compilation| {
      let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
        Either::A(str) => std::iter::once(str).map(Into::into).collect(),
        Either::B(vec) => vec.into_iter().map(Into::into).collect(),
      });
      Ok(
        exports_info
          .as_data(&compilation.exports_info_artifact)
          .is_module_used(runtime.as_ref()),
      )
    })
  }

  #[napi(ts_args_type = "runtime: string | string[] | undefined")]
  pub fn set_used_in_unknown_way(&mut self, js_runtime: JsRuntimeSpec) -> napi::Result<bool> {
    let exports_info = self.exports_info;
    self.with_compilation_mut(|compilation| {
      let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
        Either::A(str) => std::iter::once(str).map(Into::into).collect(),
        Either::B(vec) => vec.into_iter().map(Into::into).collect(),
      });
      Ok(
        exports_info
          .as_data_mut(&mut compilation.exports_info_artifact)
          .set_used_in_unknown_way(runtime.as_ref()),
      )
    })
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
    let exports_info = self.exports_info;
    self.with_compilation(|compilation| {
      let runtime: Option<RuntimeSpec> = js_runtime.map(|js_rt| match js_rt {
        Either::A(str) => std::iter::once(str).map(Into::into).collect(),
        Either::B(vec) => vec.into_iter().map(Into::into).collect(),
      });
      let names = match js_name {
        Either::A(s) => vec![Atom::from(s)],
        Either::B(v) => v.into_iter().map(Into::into).collect::<Vec<_>>(),
      };
      let used = exports_info
        .as_data(&compilation.exports_info_artifact)
        .get_used(&compilation.exports_info_artifact, &names, runtime.as_ref());
      Ok(used as u32)
    })
  }
}

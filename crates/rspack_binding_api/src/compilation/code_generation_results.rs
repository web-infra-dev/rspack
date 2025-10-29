use napi::Either;
use rspack_core::{Reflector, WeakBindingCell};
use rustc_hash::FxHashMap;

use crate::{module::ModuleObjectRef, runtime::JsRuntimeSpec, source::JsSourceToJs};

// Map<string, Source>
#[napi]
pub struct Sources {
  i: WeakBindingCell<FxHashMap<rspack_core::SourceType, rspack_core::rspack_sources::BoxSource>>,
}

impl Sources {
  pub fn new(
    i: WeakBindingCell<FxHashMap<rspack_core::SourceType, rspack_core::rspack_sources::BoxSource>>,
  ) -> Self {
    Self { i }
  }

  fn with_ref<T>(
    &self,
    f: impl FnOnce(
      &FxHashMap<rspack_core::SourceType, rspack_core::rspack_sources::BoxSource>,
    ) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.i.upgrade() {
      Some(cell) => f(&cell),
      None => Err(napi::Error::from_reason(
        "Unable to access sources now. The sources has been dropped by Rust.",
      )),
    }
  }
}

#[napi]
impl Sources {
  #[napi(js_name = "_get", ts_return_type = "JsSource | null")]
  pub fn get(&self, source_type: String) -> napi::Result<Option<JsSourceToJs>> {
    let source_type = rspack_core::SourceType::from(source_type.as_str());
    self.with_ref(|sources| match sources.get(&source_type) {
      Some(source) => {
        let source = source.as_ref().try_into()?;
        Ok(Some(source))
      }
      None => Ok(None),
    })
  }
}

#[napi]
pub struct CodeGenerationResult {
  i: WeakBindingCell<rspack_core::CodeGenerationResult>,
}

impl CodeGenerationResult {
  pub fn new(i: WeakBindingCell<rspack_core::CodeGenerationResult>) -> Self {
    Self { i }
  }

  fn with_ref<T>(
    &self,
    f: impl FnOnce(&rspack_core::CodeGenerationResult) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.i.upgrade() {
      Some(cell) => f(&cell),
      None => Err(napi::Error::from_reason(
        "Unable to access sources now. The sources has been dropped by Rust.",
      )),
    }
  }
}

#[napi]
impl CodeGenerationResult {
  #[napi(getter, ts_return_type = "Sources")]
  pub fn sources(&self) -> napi::Result<Reflector> {
    self.with_ref(|i| Ok(i.inner.reflector()))
  }
}

#[napi]
pub struct CodeGenerationResults {
  i: WeakBindingCell<rspack_core::CodeGenerationResults>,
}

impl CodeGenerationResults {
  pub fn with_ref<T>(
    &self,
    f: impl FnOnce(&rspack_core::CodeGenerationResults) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.i.upgrade() {
      Some(cell) => f(&cell),
      None => Err(napi::Error::from_reason(
        "Unable to access codeGenerationResults now. The codeGenerationResults has been dropped by Rust.",
      )),
    }
  }
}

#[napi]
impl CodeGenerationResults {
  pub fn new(i: WeakBindingCell<rspack_core::CodeGenerationResults>) -> Self {
    Self { i }
  }

  #[napi(
    ts_args_type = "module: Module, runtime: string | string[] | undefined",
    ts_return_type = "CodeGenerationResult"
  )]
  pub fn get(&self, module: ModuleObjectRef, runtime: JsRuntimeSpec) -> napi::Result<Reflector> {
    self.with_ref(|code_generation_results| {
      let rt: Option<rspack_core::RuntimeSpec> = runtime.map(|val| match val {
        Either::A(str) => std::iter::once(str).map(Into::into).collect(),
        Either::B(vec) => vec.into_iter().map(Into::into).collect(),
      });

      let code_generation_result = code_generation_results.get(&module.identifier, rt.as_ref());
      Ok(code_generation_result.reflector())
    })
  }
}

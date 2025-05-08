use napi::bindgen_prelude::WeakReference;
use rspack_core::WeakBindingCell;
use rustc_hash::FxHashMap;

use super::JsCompilation;
use crate::{JsCompatSourceOwned, JsRuntimeSpec, ModuleObjectRef, ToJsCompatSourceOwned};

pub struct Sources {
  i: WeakBindingCell<FxHashMap<rspack_core::SourceType, rspack_core::rspack_sources::BoxSource>>,
}

impl Sources {
  pub fn with_ref<T>(
    &self,
    f: impl FnOnce(
      &FxHashMap<rspack_core::SourceType, rspack_core::rspack_sources::BoxSource>,
    ) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.i.upgrade() {
      Some(cell) => f(&*cell),
      None => Err(napi::Error::from_reason(
        "Unable to access sources now. The sources has been dropped by Rust.",
      )),
    }
  }
}

impl Sources {
  pub fn get(&self, source_type: String) -> napi::Result<Option<JsCompatSourceOwned>> {
    let source_type = rspack_core::SourceType::from(source_type.as_str());
    self.with_ref(|sources| match sources.get(&source_type) {
      Some(source) => {
        let source = ToJsCompatSourceOwned::to_js_compat_source_owned(source.as_ref())?;
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
}

#[napi]
pub struct CodeGenerationResults {
  compilation_reference: WeakReference<JsCompilation>,
}

impl CodeGenerationResults {
  pub fn with_ref<T>(
    &self,
    f: impl FnOnce(&rspack_core::Compilation) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.compilation_reference.get() {
      Some(reference) => f(reference.as_ref()?),
      None => Err(napi::Error::from_reason(
        "Unable to access compilation.codeGenerationResults now. The Compilation has been garbage collected by JavaScript."
      )),
    }
  }
}

#[napi]
impl CodeGenerationResults {
  pub fn new(compilation_reference: WeakReference<JsCompilation>) -> Self {
    Self {
      compilation_reference,
    }
  }

  pub fn get(
    &self,
    module: ModuleObjectRef,
    runtime: JsRuntimeSpec,
  ) -> napi::Result<Option<rspack_core::CodeGenerationResult>> {
    self.with_ref(|compilation| {
      //   let a = compilation
      //     .code_generation_results
      //     .get(&module.identifier, Some(&runtime));

      todo!()
    })
  }
}

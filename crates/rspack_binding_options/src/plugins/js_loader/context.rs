use std::cell::RefCell;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_binding_values::{JsModule, JsResourceData, ToJsModule};
use rspack_core::{LoaderContext, LoaderContextId, RunnerContext};
use rspack_loader_runner::{LoaderItem, State as LoaderState};
use rspack_napi::Ref;
use rustc_hash::FxHashMap as HashMap;

#[napi(object)]
pub struct JsLoaderItem {
  pub request: String,
  pub r#type: String,

  // data
  pub data: serde_json::Value,

  // status
  pub normal_executed: bool,
  pub pitch_executed: bool,
}

impl From<&LoaderItem<RunnerContext>> for JsLoaderItem {
  fn from(value: &LoaderItem<RunnerContext>) -> Self {
    JsLoaderItem {
      request: value.request().to_string(),
      r#type: value.r#type().to_string(),

      data: value.data().clone(),
      normal_executed: value.normal_executed(),
      pitch_executed: value.pitch_executed(),
    }
  }
}

#[napi(string_enum)]
pub enum JsLoaderState {
  Pitching,
  Normal,
}

impl From<LoaderState> for JsLoaderState {
  fn from(value: LoaderState) -> Self {
    match value {
      LoaderState::Init | LoaderState::ProcessResource | LoaderState::Finished => {
        panic!("Unexpected loader runner state: {value:?}")
      }
      LoaderState::Pitching => JsLoaderState::Pitching,
      LoaderState::Normal => JsLoaderState::Normal,
    }
  }
}

#[napi]
pub struct JsLoaderContext(pub(crate) &'static mut LoaderContext<RunnerContext>);

#[napi]
impl JsLoaderContext {
  #[napi(getter)]
  pub fn resource_data(&self) -> JsResourceData {
    self.0.resource_data.as_ref().into()
  }

  #[napi(getter, js_name = "_moduleIdentifier")]
  pub fn module_identifier(&self) -> &str {
    self.0.context.module.module_identifier.as_str()
  }

  #[napi(getter, js_name = "_module")]
  pub fn module(&self) -> JsModule {
    self
      .0
      .context
      .module
      .to_js_module()
      .expect("CompilerModuleContext::to_js_module should not fail.")
  }

  #[napi(getter)]
  pub fn hot(&self) -> bool {
    self.0.hot
  }

  #[napi(getter)]
  pub fn content(&self) -> napi::Either<Null, Buffer> {
    match &self.0.content {
      Some(c) => napi::Either::B(c.to_owned().into_bytes().into()),
      None => napi::Either::A(Null),
    }
  }

  #[napi(setter)]
  pub fn set_content(&mut self, val: napi::Either<Null, Buffer>) {
    self.0.content = match val {
      napi::Either::A(_) => None,
      napi::Either::B(b) => Some(rspack_core::Content::from(Into::<Vec<u8>>::into(b))),
    }
  }

  #[napi(getter)]
  pub fn source_map(&self) -> Result<Either<Buffer, ()>> {
    match &self.0.source_map {
      Some(v) => {
        let s = v
          .clone()
          .to_json()
          .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(Either::A(s.into_bytes().into()))
      }
      None => Ok(Either::B(())),
    }
  }

  #[napi(setter)]
  pub fn set_source_map(&mut self, val: napi::Either<Buffer, ()>) -> Result<()> {
    self.0.source_map = match val {
      napi::Either::A(val) => {
        let source_map = rspack_core::rspack_sources::SourceMap::from_slice(&val.to_vec())
          .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Some(source_map)
      }
      napi::Either::B(_) => None,
    };
    Ok(())
  }

  #[napi(getter)]
  pub fn cacheable(&self) -> bool {
    self.0.cacheable
  }

  #[napi(setter)]
  pub fn set_cacheable(&mut self, val: bool) {
    self.0.cacheable = val;
  }

  #[napi(getter)]
  pub fn loader_items(&self) -> Vec<JsLoaderItem> {
    self.0.loader_items.iter().map(Into::into).collect()
  }

  #[napi(setter)]
  pub fn set_loader_items(&mut self, mut val: Vec<JsLoaderItem>) {
    self.0.loader_items = self
      .0
      .loader_items
      .drain(..)
      .zip(val.drain(..))
      .map(|(mut to, from)| {
        if from.normal_executed {
          to.set_normal_executed()
        }
        if from.pitch_executed {
          to.set_pitch_executed()
        }
        to.set_data(from.data);
        to
      })
      .collect();
  }

  #[napi(getter)]
  pub fn loader_index(&self) -> i32 {
    self.0.loader_index
  }

  #[napi(setter)]
  pub fn set_loader_index(&mut self, val: i32) {
    self.0.loader_index = val;
  }

  #[napi(getter)]
  pub fn loader_state(&self) -> JsLoaderState {
    self.0.state().into()
  }

  #[napi]
  pub fn add_dependency(&mut self, file: String) {
    self.0.file_dependencies.insert(file.into());
  }

  #[napi]
  pub fn add_context_dependency(&mut self, file: String) {
    self.0.context_dependencies.insert(file.into());
  }

  #[napi]
  pub fn add_missing_dependency(&mut self, file: String) {
    self.0.missing_dependencies.insert(file.into());
  }

  #[napi]
  pub fn add_build_dependency(&mut self, file: String) {
    self.0.build_dependencies.insert(file.into());
  }

  #[napi]
  pub fn get_dependencies(&self) -> Vec<String> {
    self
      .0
      .file_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi]
  pub fn get_context_dependencies(&self) -> Vec<String> {
    self
      .0
      .context_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi]
  pub fn get_missing_dependencies(&self) -> Vec<String> {
    self
      .0
      .missing_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi]
  pub fn clear_dependencies(&mut self) {
    self.0.file_dependencies.clear();
    self.0.context_dependencies.clear();
    self.0.build_dependencies.clear();
    self.0.cacheable = true;
  }
}

thread_local! {
  pub static LOADER_CONTEXT_INSTANCE_REFS: RefCell<HashMap<LoaderContextId, Ref>> = Default::default();
}

pub struct JsLoaderContextWrapper(&'static mut LoaderContext<RunnerContext>);

impl JsLoaderContextWrapper {
  pub fn new(value: &mut LoaderContext<RunnerContext>) -> Self {
    let context = unsafe {
      std::mem::transmute::<
        &'_ mut LoaderContext<RunnerContext>,
        &'static mut LoaderContext<RunnerContext>,
      >(value)
    };
    Self(context)
  }
}

impl ToNapiValue for JsLoaderContextWrapper {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    LOADER_CONTEXT_INSTANCE_REFS.with(|refs| {
      let mut refs = refs.borrow_mut();
      match refs.entry(val.0.id) {
        std::collections::hash_map::Entry::Occupied(entry) => {
          let r = entry.get();
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          let env_wrapper = Env::from_raw(env);
          let instance = JsLoaderContext(val.0).into_instance(env_wrapper)?;
          let napi_value = ToNapiValue::to_napi_value(env, instance)?;
          let r = Ref::new(env, napi_value, 1)?;
          let r = entry.insert(r);
          ToNapiValue::to_napi_value(env, r)
        }
      }
    })
  }
}

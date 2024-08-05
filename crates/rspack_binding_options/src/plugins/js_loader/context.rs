use napi::bindgen_prelude::*;
use napi_derive::napi;
use once_cell::sync::OnceCell;
use rspack_binding_values::{JsModule, JsResourceData, ToJsModule};
use rspack_core::{LoaderContext, RunnerContext};
use rspack_loader_runner::{LoaderItem, State as LoaderState};
use rspack_napi::Ref;

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
  pub fn file_dependencies(&self) -> Vec<String> {
    self
      .0
      .file_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi(setter)]
  pub fn set_file_dependencies(&mut self, val: Vec<String>) {
    self.0.file_dependencies = val.into_iter().map(std::path::PathBuf::from).collect();
  }

  #[napi(getter)]
  pub fn context_dependencies(&self) -> Vec<String> {
    self
      .0
      .context_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi(setter)]
  pub fn set_context_dependencies(&mut self, val: Vec<String>) {
    self.0.context_dependencies = val.into_iter().map(std::path::PathBuf::from).collect();
  }

  #[napi(getter)]
  pub fn missing_dependencies(&self) -> Vec<String> {
    self
      .0
      .missing_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi(setter)]
  pub fn set_missing_dependencies(&mut self, val: Vec<String>) {
    self.0.missing_dependencies = val.into_iter().map(std::path::PathBuf::from).collect();
  }

  #[napi(getter)]
  pub fn build_dependencies(&self) -> Vec<String> {
    self
      .0
      .build_dependencies
      .iter()
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi(setter)]
  pub fn set_build_dependencies(&mut self, val: Vec<String>) {
    self.0.build_dependencies = val.into_iter().map(std::path::PathBuf::from).collect();
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
}

pub struct JsLoaderContextInstance {
  class: Option<JsLoaderContext>,
  instance: OnceCell<Ref>,
}

impl ToNapiValue for JsLoaderContextInstance {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let instance_ref = val.instance.get_or_try_init(|| {
      let napi_val = ToNapiValue::to_napi_value(
        env,
        val
          .class
          .expect("If instance not initialized, class must be set")
          .into_instance(Env::from_raw(env)),
      )?;
      Ref::new(env, napi_val, 1)
    })?;
    ToNapiValue::to_napi_value(env, instance_ref)
  }
}

impl FromNapiValue for JsLoaderContextInstance {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let r = Ref::new(env, napi_val, 1)?;
    Ok(Self {
      class: None,
      instance: OnceCell::with_value(r),
    })
  }
}

impl From<&mut LoaderContext<RunnerContext>> for JsLoaderContextInstance {
  fn from(value: &mut LoaderContext<RunnerContext>) -> Self {
    let class = JsLoaderContext(unsafe {
      std::mem::transmute::<
        &'_ mut LoaderContext<RunnerContext>,
        &'static mut LoaderContext<RunnerContext>,
      >(value)
    });
    JsLoaderContextInstance {
      class: Some(class),
      instance: OnceCell::default(),
    }
  }
}

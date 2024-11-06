use std::{cell::RefCell, ptr::NonNull};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_binding_values::{JsModuleWrapper, JsResourceData};
use rspack_core::{AdditionalData, LoaderContext, LoaderContextId, RunnerContext};
use rspack_loader_runner::{LoaderItem, State as LoaderState};
use rspack_napi::{threadsafe_js_value_ref::ThreadsafeJsValueRef, OneShotRef};
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
pub struct JsLoaderContext {
  id: LoaderContextId,
  pub(crate) inner: NonNull<LoaderContext<RunnerContext>>,
}

impl JsLoaderContext {
  fn as_ref(&self) -> napi::Result<&'static LoaderContext<RunnerContext>> {
    let context = unsafe { self.inner.as_ref() };
    if context.id == self.id {
      return Ok(context);
    }

    Err(napi::Error::from_reason(format!(
      "Unable to access loader context with id = {:?} now. The loader context have been removed on the Rust side.",
      self.id
    )))
  }

  fn as_mut(&mut self) -> napi::Result<&'static mut LoaderContext<RunnerContext>> {
    let context = unsafe { self.inner.as_mut() };
    if context.id == self.id {
      return Ok(context);
    }

    Err(napi::Error::from_reason(format!(
      "Unable to access loader context with id = {:?} now. The loader context have been removed on the Rust side.",
      self.id
    )))
  }
}

#[napi]
impl JsLoaderContext {
  #[napi(getter, ts_return_type = "any")]
  pub fn additional_data(&self) -> Result<Option<ThreadsafeJsValueRef<Unknown>>> {
    let context = self.as_ref()?;

    Ok(
      context
        .additional_data
        .as_ref()
        .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
        .cloned(),
    )
  }

  #[napi(setter, ts_args_type = "val: any")]
  pub fn set_additional_data(&mut self, val: Option<ThreadsafeJsValueRef<Unknown>>) -> Result<()> {
    let context = self.as_mut()?;

    context.additional_data = if let Some(val) = val {
      let mut additional = AdditionalData::default();
      additional.insert(val);
      Some(additional)
    } else {
      None
    };
    Ok(())
  }

  #[napi(getter)]
  pub fn resource_data(&self) -> Result<JsResourceData> {
    let context = self.as_ref()?;

    Ok(context.resource_data.as_ref().into())
  }

  #[napi(getter, js_name = "_module", ts_return_type = "JsModule")]
  pub fn module(&self) -> Result<JsModuleWrapper> {
    let context = self.as_ref()?;

    let module = unsafe { context.context.module.as_ref() };
    Ok(JsModuleWrapper::new(
      module,
      context.context.compilation_id,
      None,
    ))
  }

  #[napi(getter)]
  pub fn hot(&self) -> Result<bool> {
    let context = self.as_ref()?;

    Ok(context.hot)
  }

  #[napi(getter)]
  pub fn content(&self) -> Result<Either3<Null, Buffer, &String>> {
    let context = self.as_ref()?;

    Ok(match &context.content {
      Some(c) => match c {
        rspack_core::Content::String(s) => Either3::C(s),
        rspack_core::Content::Buffer(b) => Either3::B(b.to_owned().into()),
      },
      None => Either3::A(Null),
    })
  }

  #[napi(setter)]
  pub fn set_content(&mut self, val: Either3<Null, Buffer, String>) -> Result<()> {
    let context = self.as_mut()?;

    context.content = match val {
      Either3::A(_) => None,
      Either3::B(b) => Some(rspack_core::Content::from(Into::<Vec<u8>>::into(b))),
      Either3::C(s) => Some(rspack_core::Content::from(s)),
    };
    Ok(())
  }

  #[napi(getter)]
  pub fn source_map(&self) -> Result<Either<String, ()>> {
    let context = self.as_ref()?;

    Ok(match &context.source_map {
      Some(v) => {
        let s = v
          .clone()
          .to_json()
          .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Either::A(s)
      }
      None => Either::B(()),
    })
  }

  #[napi(setter)]
  pub fn set_source_map(&mut self, val: napi::Either<String, ()>) -> Result<()> {
    let context = self.as_mut()?;

    context.source_map = match val {
      napi::Either::A(val) => {
        let source_map = rspack_core::rspack_sources::SourceMap::from_slice(val.as_bytes())
          .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Some(source_map)
      }
      napi::Either::B(_) => None,
    };
    Ok(())
  }

  #[napi(getter)]
  pub fn cacheable(&self) -> Result<bool> {
    let context = self.as_ref()?;

    Ok(context.cacheable)
  }

  #[napi(setter)]
  pub fn set_cacheable(&mut self, val: bool) -> Result<()> {
    let context = self.as_mut()?;

    context.cacheable = val;
    Ok(())
  }

  #[napi(getter)]
  pub fn loader_items(&self) -> Result<Vec<JsLoaderItem>> {
    let context = self.as_ref()?;

    Ok(context.loader_items.iter().map(Into::into).collect())
  }

  #[napi(setter)]
  pub fn set_loader_items(&mut self, mut val: Vec<JsLoaderItem>) -> Result<()> {
    let context = self.as_mut()?;

    context.loader_items = context
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
        // JS loader should always be considered as finished
        to.set_finish_called();
        to
      })
      .collect();
    Ok(())
  }

  #[napi(getter)]
  pub fn loader_index(&self) -> Result<i32> {
    let context = self.as_ref()?;

    Ok(context.loader_index)
  }

  #[napi(setter)]
  pub fn set_loader_index(&mut self, val: i32) -> Result<()> {
    let context = self.as_mut()?;

    context.loader_index = val;
    Ok(())
  }

  #[napi(getter)]
  pub fn loader_state(&self) -> Result<JsLoaderState> {
    let context = self.as_ref()?;

    Ok(context.state().into())
  }

  #[napi(js_name = "__internal__addParseMeta")]
  pub fn add_parse_meta(&mut self, key: String, val: String) -> Result<()> {
    let context = self.as_mut()?;

    context.parse_meta.insert(key, val);
    Ok(())
  }

  #[napi]
  pub fn add_dependency(&mut self, file: String) -> Result<()> {
    let context = self.as_mut()?;

    context.file_dependencies.insert(file.into());
    Ok(())
  }

  #[napi]
  pub fn add_context_dependency(&mut self, file: String) -> Result<()> {
    let context = self.as_mut()?;

    context.context_dependencies.insert(file.into());
    Ok(())
  }

  #[napi]
  pub fn add_missing_dependency(&mut self, file: String) -> Result<()> {
    let context = self.as_mut()?;

    context.missing_dependencies.insert(file.into());
    Ok(())
  }

  #[napi]
  pub fn add_build_dependency(&mut self, file: String) -> Result<()> {
    let context = self.as_mut()?;

    context.build_dependencies.insert(file.into());
    Ok(())
  }

  #[napi]
  pub fn get_dependencies(&self) -> Result<Vec<String>> {
    let context = self.as_ref()?;

    Ok(
      context
        .file_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
    )
  }

  #[napi]
  pub fn get_context_dependencies(&self) -> Result<Vec<String>> {
    let context = self.as_ref()?;

    Ok(
      context
        .context_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
    )
  }

  #[napi]
  pub fn get_missing_dependencies(&self) -> Result<Vec<String>> {
    let context = self.as_ref()?;

    Ok(
      context
        .missing_dependencies
        .iter()
        .map(|i| i.to_string_lossy().to_string())
        .collect(),
    )
  }

  #[napi]
  pub fn clear_dependencies(&mut self) -> Result<()> {
    let context = self.as_mut()?;

    context.file_dependencies.clear();
    context.context_dependencies.clear();
    context.build_dependencies.clear();
    context.cacheable = true;
    Ok(())
  }
}

thread_local! {
  pub static LOADER_CONTEXT_INSTANCE_REFS: RefCell<HashMap<LoaderContextId, OneShotRef<JsLoaderContext>>> = Default::default();
}

pub struct JsLoaderContextWrapper {
  id: LoaderContextId,
  inner: NonNull<LoaderContext<RunnerContext>>,
}

unsafe impl Send for JsLoaderContextWrapper {}

impl JsLoaderContextWrapper {
  pub fn new(value: &LoaderContext<RunnerContext>) -> Self {
    #[allow(clippy::unwrap_used)]
    Self {
      id: value.id,
      inner: NonNull::new(
        value as *const LoaderContext<RunnerContext> as *mut LoaderContext<RunnerContext>,
      )
      .unwrap(),
    }
  }
}

impl ToNapiValue for JsLoaderContextWrapper {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    LOADER_CONTEXT_INSTANCE_REFS.with(|refs| {
      let mut refs = refs.borrow_mut();
      match refs.entry(val.id) {
        std::collections::hash_map::Entry::Occupied(entry) => {
          let r = entry.get();
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          let js_context = JsLoaderContext {
            id: val.id,
            inner: val.inner,
          };
          let r = OneShotRef::new(env, js_context)?;
          let r = entry.insert(r);
          ToNapiValue::to_napi_value(env, r)
        }
      }
    })
  }
}

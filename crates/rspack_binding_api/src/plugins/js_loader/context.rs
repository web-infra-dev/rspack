use std::{path::PathBuf, ptr::NonNull, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{Content, LoaderContext, Module, RunnerContext};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_loader_runner::State as LoaderState;
use rspack_napi::threadsafe_js_value_ref::ThreadsafeJsValueRef;
use rustc_hash::FxHashSet as HashSet;

use crate::{error::RspackError, module::ModuleObject};

#[napi(object)]
#[derive(Clone, Hash)]
pub struct JsLoaderItem {
  pub loader: String,
  pub r#type: String,

  // data
  pub data: serde_json::Value,

  // status
  pub normal_executed: bool,
  pub pitch_executed: bool,

  pub no_pitch: bool,
}

impl From<&rspack_loader_runner::LoaderItem<RunnerContext>> for JsLoaderItem {
  fn from(value: &rspack_loader_runner::LoaderItem<RunnerContext>) -> Self {
    JsLoaderItem {
      loader: value.request().to_string(),
      r#type: value.r#type().to_string(),

      data: value.data().clone(),
      normal_executed: value.normal_executed(),
      pitch_executed: value.pitch_executed(),

      no_pitch: false,
    }
  }
}

impl<C> From<&Arc<dyn rspack_core::Loader<C>>> for JsLoaderItem
where
  C: Send,
{
  fn from(loader: &Arc<dyn rspack_core::Loader<C>>) -> Self {
    let identifier = loader.identifier();

    if let Some((r#type, ident)) = identifier.split_once('|') {
      return Self {
        loader: ident.to_string(),
        data: serde_json::Value::Null,
        r#type: r#type.to_string(),
        pitch_executed: false,
        normal_executed: false,
        no_pitch: false,
      };
    }
    Self {
      loader: identifier.to_string(),
      data: serde_json::Value::Null,
      r#type: String::default(),
      pitch_executed: false,
      normal_executed: false,
      no_pitch: false,
    }
  }
}

#[napi(string_enum)]
#[derive(Clone, Copy)]
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
  /// Content maybe empty in pitching stage
  content: Either3<String, Buffer, Null>,
  additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  source_map: Option<Buffer>,
  loader_items: Vec<JsLoaderItem>,
  error: Option<RspackError>,
  utf8_hint: Option<bool>,
  loader_context: NonNull<LoaderContext<RunnerContext>>,
  active: bool,
}

pub struct JsLoaderContextOutput {
  pub content: Either3<String, Buffer, Null>,
  pub additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  pub source_map: Option<Buffer>,
  pub loader_items: Vec<JsLoaderItem>,
  pub error: Option<RspackError>,
  pub utf8_hint: Option<bool>,
}

struct JsLoaderContextValueProperties {
  content: Either3<String, Buffer, Null>,
  additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  source_map: Option<Buffer>,
  loader_items: Vec<JsLoaderItem>,
  error: Option<RspackError>,
  utf8_hint: Option<bool>,
}

fn dirname(path: &str) -> String {
  if path == "/" {
    return "/".to_string();
  }
  let i = path.rfind('/');
  let j = path.rfind('\\');
  let i2 = path.find('/');
  let j2 = path.find('\\');
  let idx = match (i, j) {
    (Some(i), Some(j)) => Some(i.max(j)),
    (Some(i), None) => Some(i),
    (None, Some(j)) => Some(j),
    (None, None) => None,
  };
  let idx2 = match (i, j, i2, j2) {
    (Some(i), Some(j), Some(i2), Some(j2)) => Some(if i > j { i2 } else { j2 }),
    (Some(_), None, Some(i2), _) => Some(i2),
    (None, Some(_), _, Some(j2)) => Some(j2),
    _ => None,
  };

  let Some(idx) = idx else {
    return path.to_string();
  };
  if Some(idx) == idx2 {
    path[..idx + 1].to_string()
  } else {
    path[..idx].to_string()
  }
}

fn dependencies_to_strings(dependencies: &HashSet<PathBuf>) -> Vec<String> {
  dependencies
    .iter()
    .map(|i| i.to_string_lossy().to_string())
    .collect()
}

// SAFETY: `JsLoaderContext` is sent to the Node.js thread while `loader_yield` is
// suspended on the returned promise. No Rust code touches the pointed
// `LoaderContext` until the promise is resolved and the context is deactivated.
unsafe impl Send for JsLoaderContext {}

#[napi]
impl JsLoaderContext {
  #[napi(getter, ts_return_type = "string | Buffer | null")]
  pub fn content(&mut self) -> napi::Result<Either3<String, Buffer, Null>> {
    self.ensure_active()?;
    Ok(std::mem::replace(&mut self.content, Either3::C(Null)))
  }

  #[napi(setter, ts_args_type = "content: string | Buffer | null")]
  pub fn set_content(&mut self, content: Either3<String, Buffer, Null>) -> napi::Result<()> {
    self.ensure_active()?;
    self.content = content;
    Ok(())
  }

  #[napi(getter, ts_return_type = "any")]
  pub fn additional_data(
    &mut self,
  ) -> napi::Result<Option<ThreadsafeJsValueRef<Unknown<'static>>>> {
    self.ensure_active()?;
    Ok(self.additional_data.take())
  }

  #[napi(setter, ts_args_type = "additionalData: any")]
  pub fn set_additional_data(
    &mut self,
    additional_data: Option<ThreadsafeJsValueRef<Unknown<'static>>>,
  ) -> napi::Result<()> {
    self.ensure_active()?;
    self.additional_data = additional_data;
    Ok(())
  }

  #[napi(getter)]
  pub fn source_map(&mut self) -> napi::Result<Option<Buffer>> {
    self.ensure_active()?;
    Ok(self.source_map.take())
  }

  #[napi(setter)]
  pub fn set_source_map(&mut self, source_map: Option<Buffer>) -> napi::Result<()> {
    self.ensure_active()?;
    self.source_map = source_map;
    Ok(())
  }

  #[napi(getter)]
  pub fn loader_items(&mut self) -> napi::Result<Vec<JsLoaderItem>> {
    self.ensure_active()?;
    Ok(std::mem::take(&mut self.loader_items))
  }

  #[napi(setter)]
  pub fn set_loader_items(&mut self, loader_items: Vec<JsLoaderItem>) -> napi::Result<()> {
    self.ensure_active()?;
    self.loader_items = loader_items;
    Ok(())
  }

  #[napi(setter, js_name = "__internal__error")]
  pub fn set_error(&mut self, error: Option<RspackError>) -> napi::Result<()> {
    self.ensure_active()?;
    self.error = error;
    Ok(())
  }

  #[napi(setter, js_name = "__internal__utf8Hint")]
  pub fn set_utf8_hint(&mut self, utf8_hint: Option<bool>) -> napi::Result<()> {
    self.ensure_active()?;
    self.utf8_hint = utf8_hint;
    Ok(())
  }

  #[napi(getter)]
  pub fn resource(&self) -> napi::Result<String> {
    Ok(self.loader_context()?.resource_data.resource().to_owned())
  }

  #[napi(getter, js_name = "_module", ts_return_type = "Module")]
  pub fn module(&self) -> napi::Result<ModuleObject> {
    let cx = self.loader_context()?;
    let module = &cx.context.module;
    Ok(ModuleObject::with_ptr(
      NonNull::new(module.as_ref() as *const dyn Module as *mut dyn Module).ok_or_else(|| {
        napi::Error::from_reason("Failed to create Module object for loader context")
      })?,
      cx.context.compiler_id,
    ))
  }

  #[napi(getter, ts_return_type = "Readonly<boolean>")]
  pub fn hot(&self) -> napi::Result<bool> {
    Ok(self.loader_context()?.hot)
  }

  #[napi(getter)]
  pub fn resource_path(&self) -> napi::Result<Option<String>> {
    Ok(
      self
        .loader_context()?
        .resource_data
        .path()
        .map(|p| p.as_str().to_string()),
    )
  }

  #[napi(getter)]
  pub fn resource_query(&self) -> napi::Result<Option<String>> {
    Ok(
      self
        .loader_context()?
        .resource_data
        .query()
        .map(ToOwned::to_owned),
    )
  }

  #[napi(getter)]
  pub fn resource_fragment(&self) -> napi::Result<Option<String>> {
    Ok(
      self
        .loader_context()?
        .resource_data
        .fragment()
        .map(ToOwned::to_owned),
    )
  }

  #[napi(getter)]
  pub fn context(&self) -> napi::Result<Option<String>> {
    Ok(
      self
        .loader_context()?
        .resource_data
        .path()
        .map(|p| dirname(p.as_str())),
    )
  }

  #[napi(getter)]
  pub fn loader_index(&self) -> napi::Result<i32> {
    Ok(self.loader_context()?.loader_index)
  }

  #[napi(setter)]
  pub fn set_loader_index(&mut self, loader_index: i32) -> napi::Result<()> {
    self.loader_context_mut()?.loader_index = loader_index;
    Ok(())
  }

  #[napi(getter, ts_return_type = "Readonly<JsLoaderState>")]
  pub fn loader_state(&self) -> napi::Result<JsLoaderState> {
    Ok(self.loader_context()?.state().into())
  }

  #[napi]
  pub fn add_dependency(&mut self, file: String) -> napi::Result<()> {
    self
      .loader_context_mut()?
      .file_dependencies
      .insert(file.into());
    Ok(())
  }

  #[napi]
  pub fn add_context_dependency(&mut self, context: String) -> napi::Result<()> {
    self
      .loader_context_mut()?
      .context_dependencies
      .insert(context.into());
    Ok(())
  }

  #[napi]
  pub fn add_missing_dependency(&mut self, missing: String) -> napi::Result<()> {
    self
      .loader_context_mut()?
      .missing_dependencies
      .insert(missing.into());
    Ok(())
  }

  #[napi]
  pub fn add_build_dependency(&mut self, file: String) -> napi::Result<()> {
    self
      .loader_context_mut()?
      .build_dependencies
      .insert(file.into());
    Ok(())
  }

  #[napi]
  pub fn get_dependencies(&self) -> napi::Result<Vec<String>> {
    Ok(dependencies_to_strings(
      &self.loader_context()?.file_dependencies,
    ))
  }

  #[napi]
  pub fn get_context_dependencies(&self) -> napi::Result<Vec<String>> {
    Ok(dependencies_to_strings(
      &self.loader_context()?.context_dependencies,
    ))
  }

  #[napi]
  pub fn get_missing_dependencies(&self) -> napi::Result<Vec<String>> {
    Ok(dependencies_to_strings(
      &self.loader_context()?.missing_dependencies,
    ))
  }

  #[napi]
  pub fn clear_dependencies(&mut self) -> napi::Result<()> {
    let cx = self.loader_context_mut()?;
    cx.file_dependencies.clear();
    cx.context_dependencies.clear();
    cx.missing_dependencies.clear();
    cx.cacheable = true;
    Ok(())
  }

  #[napi]
  pub fn set_cacheable(&mut self, cacheable: bool) -> napi::Result<()> {
    if !cacheable {
      self.loader_context_mut()?.cacheable = false;
    }
    Ok(())
  }

  #[napi(js_name = "__internal__setParseMeta")]
  pub fn set_parse_meta(&mut self, key: String, value: String) -> napi::Result<()> {
    self
      .loader_context_mut()?
      .parse_meta
      .insert(key, Box::new(value) as _);
    Ok(())
  }
}

impl JsLoaderContext {
  fn inactive_error() -> napi::Error {
    napi::Error::from_reason(
      "JsLoaderContext is no longer valid after the JavaScript loader runner has finished",
    )
  }

  fn ensure_active(&self) -> napi::Result<()> {
    if self.active {
      Ok(())
    } else {
      Err(Self::inactive_error())
    }
  }

  fn loader_context(&self) -> napi::Result<&LoaderContext<RunnerContext>> {
    self.ensure_active()?;
    // SAFETY: The pointer is created from the `loader_yield` mutable reference.
    // `loader_yield` is suspended while JS owns this context, and `active` is
    // cleared before Rust resumes using the original reference.
    Ok(unsafe { self.loader_context.as_ref() })
  }

  fn loader_context_mut(&mut self) -> napi::Result<&mut LoaderContext<RunnerContext>> {
    self.ensure_active()?;
    // SAFETY: See `loader_context`. All mutable access goes through the JS
    // loader runner while the Rust future is suspended.
    Ok(unsafe { self.loader_context.as_mut() })
  }

  fn take_output(
    &mut self,
    value_properties: JsLoaderContextValueProperties,
  ) -> JsLoaderContextOutput {
    self.active = false;
    JsLoaderContextOutput {
      content: value_properties.content,
      additional_data: value_properties.additional_data,
      source_map: value_properties.source_map,
      loader_items: value_properties.loader_items,
      error: value_properties.error,
      utf8_hint: value_properties.utf8_hint,
    }
  }
}

impl FromNapiValue for JsLoaderContextOutput {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    let object = unsafe { Object::from_napi_value(env, napi_val)? };
    let value_properties = JsLoaderContextValueProperties {
      content: object.get_named_property("content")?,
      additional_data: object.get_named_property_unchecked("additionalData")?,
      source_map: object.get_named_property("sourceMap")?,
      loader_items: object.get_named_property("loaderItems")?,
      error: object.get_named_property("__internal__error")?,
      utf8_hint: object.get_named_property("__internal__utf8Hint")?,
    };
    let mut instance = unsafe { ClassInstance::<JsLoaderContext>::from_napi_value(env, napi_val)? };
    Ok(instance.take_output(value_properties))
  }
}

impl TypeName for JsLoaderContextOutput {
  fn type_name() -> &'static str {
    "JsLoaderContext"
  }

  fn value_type() -> ValueType {
    ValueType::Object
  }
}

impl ValidateNapiValue for JsLoaderContextOutput {
  unsafe fn validate(
    env: sys::napi_env,
    napi_val: sys::napi_value,
  ) -> napi::Result<sys::napi_value> {
    unsafe { Object::validate(env, napi_val) }
  }
}

impl TryFrom<&mut LoaderContext<RunnerContext>> for JsLoaderContext {
  type Error = rspack_error::Error;

  fn try_from(
    cx: &mut rspack_core::LoaderContext<RunnerContext>,
  ) -> std::result::Result<Self, Self::Error> {
    Ok(JsLoaderContext {
      content: match cx.take_content() {
        Some(Content::String(s)) => Either3::A(s),
        Some(Content::Buffer(b)) => Either3::B(b.into()),
        None => Either3::C(Null),
      },
      additional_data: cx
        .take_additional_data()
        .as_ref()
        .and_then(|data| data.get::<ThreadsafeJsValueRef<Unknown>>())
        .cloned(),
      source_map: cx
        .take_source_map()
        .map(|v| v.to_json())
        .map(|v| v.into_bytes().into()),
      loader_items: cx.loader_items.iter().map(Into::into).collect(),
      error: None,
      utf8_hint: None,
      loader_context: NonNull::from(cx),
      active: true,
    })
  }
}

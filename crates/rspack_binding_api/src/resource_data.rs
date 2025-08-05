use std::{
  cell::RefCell,
  sync::{Arc, Weak},
};

use napi::{
  Env, JsValue,
  bindgen_prelude::{JavaScriptClassExt, JsObjectValue, ToNapiValue},
  sys::napi_value,
};
use napi_derive::napi;
use rspack_napi::{Property, define_properties};

// Converting the descriptionFileData property to a JSObject may become a performance bottleneck.
// Additionally, descriptionFileData and descriptionFilePath are rarely used, so they are exposed via getter methods and only converted to JSObject when accessed.
#[napi]
pub struct ReadonlyResourceData {
  i: Weak<rspack_core::ResourceData>,
}

impl ReadonlyResourceData {
  pub fn with_ref<R>(
    &self,
    f: impl FnOnce(&rspack_core::ResourceData) -> napi::Result<R>,
  ) -> napi::Result<R> {
    match self.i.upgrade() {
      Some(arc) => f(arc.as_ref()),
      None => Err(napi::Error::from_reason(
        "ResourceData has been dropped by Rust.",
      )),
    }
  }
}

#[napi]
impl ReadonlyResourceData {
  #[napi(getter, ts_return_type = "any")]
  pub fn description_file_data(&self, env: &Env) -> napi::Result<Option<napi_value>> {
    self.with_ref(|resource_data| {
      resource_data
        .resource_description
        .as_ref()
        .map(|desc| unsafe { ToNapiValue::to_napi_value(env.raw(), desc.json()) })
        .transpose()
    })
  }

  #[napi(getter, ts_return_type = "string")]
  pub fn description_file_path(&self, env: &Env) -> napi::Result<Option<napi_value>> {
    self.with_ref(|resource_data| {
      resource_data
        .resource_description
        .as_ref()
        .map(|data| unsafe {
          ToNapiValue::to_napi_value(env.raw(), data.path().to_string_lossy().as_ref())
        })
        .transpose()
    })
  }
}

pub struct ReadonlyResourceDataWrapper {
  i: Arc<rspack_core::ResourceData>,
}

thread_local! {
  static RESOURCE_DATA_PROPERTIES_BUFFER: RefCell<Vec<Property>> = const { RefCell::new(Vec::new()) };
}

impl ToNapiValue for ReadonlyResourceDataWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    let env_wrapper = Env::from_raw(env);

    let resource_data = val.i;
    let template = ReadonlyResourceData {
      i: Arc::downgrade(&resource_data),
    };
    let instance = template.into_instance(&env_wrapper)?;
    let mut object = instance.as_object(&env_wrapper);

    RESOURCE_DATA_PROPERTIES_BUFFER.with(|ref_cell| {
      let mut properties = ref_cell.borrow_mut();
      properties.clear();
      properties.push(
        Property::new()
          .with_utf8_name(c"resource")?
          .with_value(&env_wrapper.create_string(&resource_data.resource)?),
      );
      if let Some(path) = &resource_data.resource_path {
        properties.push(
          Property::new()
            .with_utf8_name(c"path")?
            .with_value(&env_wrapper.create_string(path.as_str())?),
        );
      }
      if let Some(query) = &resource_data.resource_query {
        properties.push(
          Property::new()
            .with_utf8_name(c"query")?
            .with_value(&env_wrapper.create_string(query)?),
        );
      }
      if let Some(fragment) = &resource_data.resource_fragment {
        properties.push(
          Property::new()
            .with_utf8_name(c"fragment")?
            .with_value(&env_wrapper.create_string(fragment)?),
        );
      }
      define_properties(&env_wrapper, &mut object, &properties)
    })?;

    Ok(object.raw())
  }
}

impl From<Arc<rspack_core::ResourceData>> for ReadonlyResourceDataWrapper {
  fn from(value: Arc<rspack_core::ResourceData>) -> Self {
    ReadonlyResourceDataWrapper { i: value }
  }
}

#[napi(object)]
pub struct JsResourceData {
  /// Resource with absolute path, query and fragment
  pub resource: String,
  /// Absolute resource path only
  pub path: Option<String>,
  /// Resource query with `?` prefix
  pub query: Option<String>,
  /// Resource fragment with `#` prefix
  pub fragment: Option<String>,
  pub description_file_data: Option<serde_json::Value>,
  pub description_file_path: Option<String>,
}

impl From<rspack_core::ResourceData> for JsResourceData {
  fn from(value: rspack_core::ResourceData) -> Self {
    let (description_file_path, description_file_data) = value
      .resource_description
      .map(|data| data.into_parts())
      .unzip();
    Self {
      resource: value.resource,
      path: value.resource_path.map(|p| p.as_str().to_string()),
      query: value.resource_query,
      fragment: value.resource_fragment,
      description_file_data: description_file_data.map(std::sync::Arc::unwrap_or_clone),
      description_file_path: description_file_path.map(|path| path.to_string_lossy().into_owned()),
    }
  }
}

impl From<&rspack_core::ResourceData> for JsResourceData {
  fn from(value: &rspack_core::ResourceData) -> Self {
    Self {
      resource: value.resource.to_owned(),
      path: value.resource_path.as_ref().map(|p| p.as_str().to_string()),
      fragment: value.resource_fragment.as_ref().map(|r| r.to_owned()),
      query: value.resource_query.as_ref().map(|r| r.to_owned()),
      description_file_data: value
        .resource_description
        .as_ref()
        .map(|data| data.json().to_owned()),
      description_file_path: value
        .resource_description
        .as_ref()
        .map(|data| data.path().to_string_lossy().into_owned()),
    }
  }
}

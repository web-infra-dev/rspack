use napi::bindgen_prelude::Object;
use napi_derive::napi;
use rspack_error::{miette::IntoDiagnostic, Error};
use rspack_plugin_context_replacement::ContextReplacementPluginOptions;
use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;

#[napi(object, object_to_js = false)]
pub struct RawContextReplacementPluginOptions {
  #[napi(ts_type = "RegExp")]
  pub resource_reg_exp: RspackRegex,
  pub new_content_resource: Option<String>,
  pub new_content_recursive: Option<bool>,
  #[napi(ts_type = "RegExp")]
  pub new_content_reg_exp: Option<RspackRegex>,
  #[napi(ts_type = "Record<string, string>")]
  pub new_content_create_context_map: Option<Object>,
  // new_content_callback
}

impl TryFrom<RawContextReplacementPluginOptions> for ContextReplacementPluginOptions {
  type Error = Error;

  fn try_from(val: RawContextReplacementPluginOptions) -> Result<Self, Self::Error> {
    let RawContextReplacementPluginOptions {
      resource_reg_exp,
      new_content_resource,
      new_content_recursive,
      new_content_reg_exp,
      new_content_create_context_map,
    } = val;

    let new_content_create_context_map = if let Some(raw) = new_content_create_context_map {
      let mut map = HashMap::default();
      let keys = Object::keys(&raw).into_diagnostic()?;
      for key in keys {
        let value = raw.get::<String>(&key).into_diagnostic()?;
        if let Some(value) = value {
          map.insert(key, value);
        }
      }
      Some(map)
    } else {
      None
    };

    Ok(Self {
      resource_reg_exp,
      new_content_resource,
      new_content_recursive,
      new_content_reg_exp,
      new_content_create_context_map,
    })
  }
}

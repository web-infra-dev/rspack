use napi::bindgen_prelude::Object;
use napi_derive::napi;
use rspack_binding_values::RawRegex;
use rspack_error::{miette::IntoDiagnostic, Error};
use rspack_plugin_context_replacement::ContextReplacementPluginOptions;
use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;

#[napi(object, object_to_js = false)]
pub struct RawContextReplacementPluginOptions {
  pub resource_reg_exp: RawRegex,
  pub new_content_resource: Option<String>,
  pub new_content_recursive: Option<bool>,
  pub new_content_reg_exp: Option<RawRegex>,
  #[napi(ts_type = "Record<string, string>")]
  pub new_content_create_context_map: Option<Object>,
  // new_content_callback
}

impl TryFrom<RawContextReplacementPluginOptions> for ContextReplacementPluginOptions {
  type Error = Error;

  fn try_from(val: RawContextReplacementPluginOptions) -> Result<Self, Self::Error> {
    let new_content_reg_exp = match val.new_content_reg_exp {
      Some(js_regex) => {
        let regex = RspackRegex::with_flags(&js_regex.source, &js_regex.flags)?;
        Some(regex)
      }
      None => None,
    };

    let new_content_create_context_map = if let Some(raw) = val.new_content_create_context_map {
      let mut map = HashMap::default();
      let keys = Object::keys(&raw).into_diagnostic()?;
      for key in keys {
        let value = raw.get::<&str, String>(&key).into_diagnostic()?;
        if let Some(value) = value {
          map.insert(key, value);
        }
      }
      Some(map)
    } else {
      None
    };

    Ok(Self {
      resource_reg_exp: RspackRegex::with_flags(
        &val.resource_reg_exp.source,
        &val.resource_reg_exp.flags,
      )?,
      new_content_resource: val.new_content_resource,
      new_content_recursive: val.new_content_recursive,
      new_content_reg_exp,
      new_content_create_context_map,
    })
  }
}

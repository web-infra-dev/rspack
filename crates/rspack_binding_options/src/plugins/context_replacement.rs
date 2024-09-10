use napi_derive::napi;
use rspack_binding_values::RawRegex;
use rspack_error::Error;
use rspack_plugin_context_replacement::ContextReplacementPluginOptions;
use rspack_regex::RspackRegex;

#[napi(object, object_to_js = false)]
pub struct RawContextReplacementPluginOptions {
  pub resource_reg_exp: RawRegex,
  pub new_content_resource: Option<String>,
  pub new_content_recursive: Option<bool>,
  pub new_content_reg_exp: Option<RawRegex>,
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
    Ok(Self {
      resource_reg_exp: RspackRegex::with_flags(
        &val.resource_reg_exp.source,
        &val.resource_reg_exp.flags,
      )?,
      new_content_resource: val.new_content_resource,
      new_content_recursive: val.new_content_recursive,
      new_content_reg_exp,
    })
  }
}

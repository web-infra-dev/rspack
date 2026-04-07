use crate::js_regex::JsRegExp;

#[napi(object, object_to_js = false)]
pub struct RawCssChunkingPluginOptions {
  pub strict: Option<bool>,
  pub min_size: Option<f64>,
  pub max_size: Option<f64>,
  #[napi(ts_type = "RegExp")]
  pub exclude: Option<JsRegExp>,
}

impl TryFrom<RawCssChunkingPluginOptions> for rspack_plugin_css_chunking::CssChunkingPluginOptions {
  type Error = rspack_error::Error;

  fn try_from(options: RawCssChunkingPluginOptions) -> Result<Self, Self::Error> {
    Ok(Self {
      strict: options.strict.unwrap_or(false),
      min_size: options.min_size,
      max_size: options.max_size,
      exclude: options.exclude.map(TryInto::try_into).transpose()?,
    })
  }
}

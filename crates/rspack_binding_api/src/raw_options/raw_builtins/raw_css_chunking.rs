use rspack_regex::RspackRegex;

#[napi(object, object_to_js = false)]
pub struct RawCssChunkingPluginOptions {
  pub strict: Option<bool>,
  pub min_size: Option<f64>,
  pub max_size: Option<f64>,
  #[napi(ts_type = "RegExp")]
  pub exclude: Option<RspackRegex>,
}

impl From<RawCssChunkingPluginOptions> for rspack_plugin_css_chunking::CssChunkingPluginOptions {
  fn from(options: RawCssChunkingPluginOptions) -> Self {
    Self {
      strict: options.strict.unwrap_or(false),
      min_size: options.min_size,
      max_size: options.max_size,
      exclude: options.exclude,
    }
  }
}

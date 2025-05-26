use rspack_regex::RspackRegex;

#[napi(object, object_to_js = false)]
pub struct CssChunkingPluginOptions {
  pub strict: Option<bool>,
  #[napi(ts_type = "RegExp")]
  pub exclude: Option<RspackRegex>,
}

impl From<CssChunkingPluginOptions> for rspack_plugin_css_chunking::CssChunkingPluginOptions {
  fn from(options: CssChunkingPluginOptions) -> Self {
    Self {
      strict: options.strict.unwrap_or(false),
      exclude: options.exclude,
    }
  }
}

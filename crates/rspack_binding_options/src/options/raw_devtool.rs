use napi_derive::napi;
use rspack_plugin_devtool::SourceMapDevToolPluginOptions;

#[derive(Debug)]
#[napi(object)]
pub struct RawSourceMapDevToolPluginOptions {
  pub filename: Option<String>,
  pub append: Option<bool>,
  pub namespace: String,
  pub columns: bool,
  pub no_sources: bool,
  pub public_path: Option<String>,
}

impl From<RawSourceMapDevToolPluginOptions> for SourceMapDevToolPluginOptions {
  fn from(value: RawSourceMapDevToolPluginOptions) -> Self {
    Self {
      filename: value.filename,
      append: value.append,
      namespace: value.namespace,
      columns: value.columns,
      no_sources: value.no_sources,
      public_path: value.public_path,
    }
  }
}

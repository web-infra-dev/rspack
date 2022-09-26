use std::collections::HashMap;

use pathdiff::diff_paths;
use rspack_core::{
  rspack_sources::{MapOptions, RawSource, SourceExt},
  Plugin, PluginContext, PluginProcessAssetsOutput, ProcessAssetsArgs,
};

#[derive(Debug, Clone)]
pub struct DevtoolPluginOptions {
  pub append: bool,
  pub namespace: String,
  pub columns: bool,
  pub no_sources: bool,
  pub public_path: Option<String>,
  pub source_root: Option<String>,
}

#[derive(Debug)]
pub struct DevtoolPlugin {
  source_mapping_url_comment: Option<String>,
  module_filename_template: String,
  namespace: String,
  columns: bool,
  no_sources: bool,
  public_path: Option<String>,
  source_root: String,
}

impl DevtoolPlugin {
  pub fn new(options: DevtoolPluginOptions) -> Self {
    Self {
      source_mapping_url_comment: options
        .append
        .then(|| "\n//# sourceMappingURL=[url]".to_string()),
      module_filename_template: "rspack://[namespace]/[resourcePath]".to_string(),
      namespace: options.namespace,
      columns: options.columns,
      no_sources: options.no_sources,
      public_path: options.public_path,
      source_root: options.source_root.unwrap_or_default(),
    }
  }
}

#[async_trait::async_trait]
impl Plugin for DevtoolPlugin {
  fn name(&self) -> &'static str {
    "devtool"
  }

  async fn process_assets(
    &self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    if !args.compilation.options.devtool.source_map() {
      return Ok(());
    }
    let mut maps = HashMap::new();
    for (filename, asset) in &args.compilation.assets {
      if let Some(map) = asset.map(&MapOptions::default()) {
        maps.insert(filename.to_owned(), map);
      }
    }
    for (filename, mut map) in maps {
      map.set_file(Some(filename.clone()));
      for source in map.sources_mut() {
        let uri = if source.starts_with('<') && source.ends_with('>') {
          &source[1..source.len() - 1] // remove '<' and '>' for swc FileName::Custom
        } else {
          &source[..]
        };
        *source = if let Some(relative_path) = diff_paths(uri, &args.compilation.options.context) {
          relative_path.display().to_string()
        } else {
          uri.to_owned()
        };
      }
      let map = map
        .to_json()
        .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?;
      args
        .compilation
        .emit_asset(filename + ".map", RawSource::from(map).boxed());
    }
    Ok(())
  }
}

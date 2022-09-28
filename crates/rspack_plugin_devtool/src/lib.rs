use hashbrown::HashMap;
use once_cell::sync::Lazy;
use pathdiff::diff_paths;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  rspack_sources::{ConcatSource, MapOptions, RawSource, SourceExt, SourceMap},
  Plugin, PluginContext, PluginProcessAssetsOutput, ProcessAssetsArgs,
};
use tracing::instrument;

static IS_CSS_FILE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.css($|\?)").unwrap());

#[derive(Debug)]
pub struct DevtoolPluginOptions {
  pub inline: bool,
  pub append: bool,
  pub namespace: String,
  pub columns: bool,
  pub no_sources: bool,
  pub public_path: Option<String>,
}

#[derive(Debug)]
pub struct DevtoolPlugin {
  inline: bool,
  source_mapping_url_comment: Option<String>,
  module_filename_template: String,
  namespace: String,
  columns: bool,
  no_sources: bool,
  public_path: Option<String>,
}

impl DevtoolPlugin {
  pub fn new(options: DevtoolPluginOptions) -> Self {
    Self {
      inline: options.inline,
      source_mapping_url_comment: options
        .append
        .then(|| "# sourceMappingURL=[url]".to_string()),
      module_filename_template: "[resourcePath]".to_string(),
      namespace: options.namespace,
      columns: options.columns,
      no_sources: options.no_sources,
      public_path: options.public_path,
    }
  }
}

#[async_trait::async_trait]
impl Plugin for DevtoolPlugin {
  fn name(&self) -> &'static str {
    "devtool"
  }

  #[instrument]
  async fn process_assets(
    &self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    if !args.compilation.options.devtool.source_map() {
      return Ok(());
    }
    let maps: HashMap<String, SourceMap> = args
      .compilation
      .assets
      .par_iter()
      .filter_map(|(filename, asset)| {
        asset
          .map(&MapOptions::new(self.columns))
          .map(|source_map| (filename.to_owned(), source_map))
      })
      .collect();
    for (filename, mut map) in maps {
      map.set_file(Some(filename.clone()));
      for source in map.sources_mut() {
        let uri = if source.starts_with('<') && source.ends_with('>') {
          &source[1..source.len() - 1] // remove '<' and '>' for swc FileName::Custom
        } else {
          &source[..]
        };
        let resource_path =
          if let Some(relative_path) = diff_paths(uri, &args.compilation.options.context) {
            relative_path.to_string_lossy().to_string()
          } else {
            uri.to_owned()
          };
        *source = self
          .module_filename_template
          .replace("[namespace]", &self.namespace)
          .replace("[resourcePath]", &resource_path);
      }
      if self.no_sources {
        for content in map.sources_content_mut() {
          *content = String::default();
        }
      }
      let current_source_mapping_url_comment =
        self.source_mapping_url_comment.as_ref().map(|comment| {
          if IS_CSS_FILE.is_match(&filename) {
            format!("\n/*{}*/", comment)
          } else {
            format!("\n//{}", comment)
          }
        });
      let mut map_buffer = Vec::new();
      map
        .to_writer(&mut map_buffer)
        .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?;
      if self.inline {
        let current_source_mapping_url_comment = current_source_mapping_url_comment
          .expect("DevToolPlugin: append can't be false when inline is true.");
        let base64 = base64::encode(&map_buffer);
        let asset = args.compilation.assets.remove(&filename).unwrap();
        let asset = ConcatSource::new([
          asset,
          RawSource::from(current_source_mapping_url_comment.replace(
            "[url]",
            &format!("data:application/json;charset=utf-8;base64,{}", base64),
          ))
          .boxed(),
        ]);
        args.compilation.emit_asset(filename, asset.boxed());
      } else {
        let source_map_filename = filename.clone() + ".map";
        if let Some(current_source_mapping_url_comment) = current_source_mapping_url_comment {
          let source_map_url = if let Some(public_path) = &self.public_path {
            public_path.clone() + &source_map_filename
          } else {
            source_map_filename.clone()
          };
          let asset = args.compilation.assets.remove(&filename).unwrap();
          let asset = ConcatSource::new([
            asset,
            RawSource::from(current_source_mapping_url_comment.replace("[url]", &source_map_url))
              .boxed(),
          ]);
          args.compilation.emit_asset(filename, asset.boxed());
        }
        args
          .compilation
          .emit_asset(source_map_filename, RawSource::from(map_buffer).boxed());
      }
    }
    Ok(())
  }
}

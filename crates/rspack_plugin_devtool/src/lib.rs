use hashbrown::HashMap;
use once_cell::sync::Lazy;
use pathdiff::diff_paths;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  rspack_sources::{ConcatSource, MapOptions, RawSource, SourceExt},
  AssetInfo, CompilationAsset, Plugin, PluginContext, PluginProcessAssetsOutput, ProcessAssetsArgs,
};
use rspack_error::{internal_error, InternalError, Result};
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

  #[instrument(skip_all)]
  async fn process_assets(
    &mut self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    if !args.compilation.options.devtool.source_map() || args.compilation.options.devtool.eval() {
      return Ok(());
    }
    let maps: HashMap<String, Vec<u8>> = args
      .compilation
      .assets
      .par_iter()
      .filter_map(|(filename, asset)| {
        asset
          .get_source()
          .map(&MapOptions::new(self.columns))
          .map(|mut map| {
            map.set_file(Some(filename.clone()));
            for source in map.sources_mut() {
              let uri = if source.starts_with('<') && source.ends_with('>') {
                &source[1..source.len() - 1] // remove '<' and '>' for swc FileName::Custom
              } else {
                &source[..]
              };
              let resource_path =
                if let Some(relative_path) = diff_paths(uri, &*args.compilation.options.context) {
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
            let mut map_buffer = Vec::new();
            map
              .to_writer(&mut map_buffer)
              .map_err(|e| rspack_error::Error::InternalError(internal_error!(e.to_string())))?;
            Ok((filename.to_owned(), map_buffer))
          })
      })
      .collect::<Result<_>>()?;
    for (filename, map_buffer) in maps {
      let current_source_mapping_url_comment =
        self.source_mapping_url_comment.as_ref().map(|comment| {
          if IS_CSS_FILE.is_match(&filename) {
            format!("\n/*{}*/", comment)
          } else {
            format!("\n//{}", comment)
          }
        });
      if self.inline {
        let current_source_mapping_url_comment = current_source_mapping_url_comment
          .expect("DevToolPlugin: append can't be false when inline is true.");
        let base64 = base64::encode(&map_buffer);
        let mut asset = args.compilation.assets.remove(&filename).unwrap();
        asset.source = ConcatSource::new([
          asset.source,
          RawSource::from(current_source_mapping_url_comment.replace(
            "[url]",
            &format!("data:application/json;charset=utf-8;base64,{}", base64),
          ))
          .boxed(),
        ])
        .boxed();
        args.compilation.emit_asset(filename, asset);
      } else {
        let source_map_filename = filename.clone() + ".map";
        if let Some(current_source_mapping_url_comment) = current_source_mapping_url_comment {
          let source_map_url = if let Some(public_path) = &self.public_path {
            public_path.clone() + &source_map_filename
          } else {
            source_map_filename.clone()
          };
          let mut asset = args.compilation.assets.remove(&filename).unwrap();
          asset.source = ConcatSource::new([
            asset.source,
            RawSource::from(current_source_mapping_url_comment.replace("[url]", &source_map_url))
              .boxed(),
          ])
          .boxed();
          asset.info.related.source_map = Some(source_map_filename.clone());
          args.compilation.emit_asset(filename, asset);
        }
        let source_map_asset_info = AssetInfo {
          development: true,
          ..Default::default()
        };
        args.compilation.emit_asset(
          source_map_filename,
          CompilationAsset::new(RawSource::from(map_buffer).boxed(), source_map_asset_info),
        );
      }
    }
    Ok(())
  }
}

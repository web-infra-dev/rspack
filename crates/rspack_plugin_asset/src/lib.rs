use std::{ffi::OsStr, path::Path};

use async_trait::async_trait;
use rayon::prelude::*;
use rspack_error::Result;

use rspack_core::{
  AssetContent, AssetParserOptions, FilenameRenderOptions, ModuleRenderResult, Plugin,
  PluginContext, PluginRenderManifestHookOutput, RenderManifestArgs, RenderManifestEntry,
  SourceType,
};

mod asset;
mod asset_source;

use asset::AssetParser;
use asset_source::AssetSourceParser;

#[derive(Debug)]
pub struct AssetConfig {
  pub parse_options: Option<AssetParserOptions>,
}
#[derive(Debug)]
pub struct AssetPlugin {
  config: AssetConfig,
}
impl AssetPlugin {
  pub fn new(config: AssetConfig) -> AssetPlugin {
    AssetPlugin { config }
  }
}

#[async_trait]
impl Plugin for AssetPlugin {
  fn name(&self) -> &'static str {
    "asset"
  }

  fn apply(
    &mut self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    ctx.context.register_parser(
      rspack_core::ModuleType::Asset,
      Box::new(AssetParser::with_auto(
        self
          .config
          .parse_options
          .as_ref()
          .and_then(|x| x.data_url_condition.clone()),
      )),
    );
    ctx.context.register_parser(
      rspack_core::ModuleType::AssetInline,
      Box::new(AssetParser::with_inline()),
    );
    ctx.context.register_parser(
      rspack_core::ModuleType::AssetResource,
      Box::new(AssetParser::with_resource()),
    );
    ctx.context.register_parser(
      rspack_core::ModuleType::AssetSource,
      Box::new(AssetSourceParser::default()),
    );

    Ok(())
  }

  fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let module_graph = &compilation.module_graph;

    let ordered_modules = compilation
      .chunk_graph
      .get_chunk_modules(&args.chunk_ukey, module_graph);

    let assets = ordered_modules
      .par_iter()
      .filter(|module| module.module.source_types().contains(&SourceType::Asset))
      .map(|module| {
        module
          .module
          .render(SourceType::Asset, module, compilation)
          .map(|result| {
            if let Some(ModuleRenderResult::Asset(asset)) = result {
              let path = Path::new(&module.id);
              Some(RenderManifestEntry::new(
                AssetContent::Buffer(asset),
                args
                  .compilation
                  .options
                  .output
                  .asset_module_filename
                  .render(FilenameRenderOptions {
                    filename: path
                      .file_stem()
                      .and_then(OsStr::to_str)
                      .map(|s| s.to_owned()),
                    extension: path
                      .extension()
                      .and_then(OsStr::to_str)
                      .map(|str| format!("{}{}", ".", str)),
                    id: None,
                    contenthash: None,
                  }),
              ))
            } else {
              None
            }
          })
      })
      .collect::<Result<Vec<Option<RenderManifestEntry>>>>()?
      .into_par_iter()
      .flatten()
      .collect::<Vec<RenderManifestEntry>>();

    Ok(assets)
  }
}

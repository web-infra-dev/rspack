use std::{ffi::OsStr, path::Path};

use anyhow::Result;
use async_trait::async_trait;
use rayon::prelude::*;

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
    let chunk = compilation
      .chunk_graph
      .chunk_by_id(args.chunk_id)
      .ok_or_else(|| anyhow::format_err!("Not found chunk {:?}", args.chunk_id))?;

    let ordered_modules = chunk.ordered_modules(module_graph);

    let assets = ordered_modules
      .par_iter()
      .filter(|module| {
        module
          .module
          .source_types(module, compilation)
          .contains(&SourceType::Asset)
      })
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
                    filename: Some(path.file_stem().and_then(OsStr::to_str).unwrap().to_owned()),
                    extension: Some(
                      path
                        .extension()
                        .and_then(OsStr::to_str)
                        .map(|str| format!("{}{}", ".", str))
                        .unwrap(),
                    ),
                    id: None,
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

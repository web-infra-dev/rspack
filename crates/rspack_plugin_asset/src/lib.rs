use std::{ffi::OsStr, path::Path};

use anyhow::Result;
use async_trait::async_trait;
use rayon::prelude::*;
use tokio::fs;

use rspack_core::{
  Asset, AssetContent, Content, Filename, LoadArgs, ModuleRenderResult, ModuleType,
  NormalModuleFactoryContext, Plugin, PluginContext, PluginLoadHookOutput,
  PluginRenderManifestHookOutput, RenderManifestArgs, SourceType,
};

mod asset;
mod asset_source;

use asset::AssetParser;
use asset_source::AssetSourceParser;

#[derive(Debug)]
pub struct AssetPlugin {}

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
      Box::new(AssetParser::with_auto()),
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

  async fn load(
    &self,
    ctx: PluginContext<&mut NormalModuleFactoryContext>,
    args: LoadArgs<'_>,
  ) -> PluginLoadHookOutput {
    if matches!(
      ctx.context.module_type,
      Some(ModuleType::Asset) | Some(ModuleType::AssetInline) | Some(ModuleType::AssetResource)
    ) {
      Ok(Some(Content::Buffer(fs::read(&args.uri).await?)))
    } else {
      Ok(None)
    }
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
              Some(Asset::new(
                AssetContent::Buffer(asset),
                args
                  .compilation
                  .options
                  .output
                  .asset_module_filename
                  .filename(
                    path.file_stem().and_then(OsStr::to_str).unwrap().to_owned(),
                    path
                      .extension()
                      .and_then(OsStr::to_str)
                      .map(|str| format!("{}{}", ".", str))
                      .unwrap(),
                  ),
              ))
            } else {
              None
            }
          })
      })
      .collect::<Result<Vec<Option<Asset>>>>()?
      .into_par_iter()
      .flatten()
      .collect::<Vec<Asset>>();

    Ok(assets)
  }
}

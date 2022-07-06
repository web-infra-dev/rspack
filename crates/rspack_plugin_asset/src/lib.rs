use std::{ffi::OsStr, path::Path};

use anyhow::Result;
use async_trait::async_trait;
use hashbrown::HashSet;
use mime_guess::MimeGuess;
use rayon::prelude::*;

use rspack_core::{
  Asset, AssetContent, BoxModule, Content, Filename, LoadArgs, Module, ModuleRenderResult,
  ModuleType, NormalModuleFactoryContext, OutputAssetModuleFilename, Parser, Plugin, PluginContext,
  PluginLoadHookOutput, PluginRenderManifestHookOutput, RenderManifestArgs, SourceType,
};

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

    Ok(())
  }

  async fn load(
    &self,
    ctx: PluginContext<&mut NormalModuleFactoryContext>,
    _args: LoadArgs<'_>,
  ) -> PluginLoadHookOutput {
    if matches!(
      ctx.context.module_type,
      Some(ModuleType::Asset) | Some(ModuleType::AssetInline) | Some(ModuleType::AssetResource)
    ) {
      Ok(Some(Content::String("".to_owned())))
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
      .filter_map(|module| {
        if let Ok(Some(ModuleRenderResult::Asset(asset))) =
          module.module.render(SourceType::Asset, module, compilation)
        {
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
      .collect::<Vec<Asset>>();

    Ok(assets)
  }
}

#[derive(Debug)]
struct AssetParser {
  data_url: Option<bool>,
}

impl AssetParser {
  fn with_auto() -> Self {
    Self { data_url: None }
  }

  fn with_inline() -> Self {
    Self {
      data_url: Some(true),
    }
  }

  fn with_resource() -> Self {
    Self {
      data_url: Some(false),
    }
  }
}

// Webpack's default parser.dataUrlCondition.maxSize
static DEFAULT_MAX_SIZE: u32 = 8096;

impl Parser for AssetParser {
  fn parse(
    &self,
    module_type: ModuleType,
    args: rspack_core::ParseModuleArgs,
  ) -> Result<BoxModule> {
    let buf = std::fs::read(&args.uri)?;
    let size = buf.len() as u32;

    let mut is_inline = size <= DEFAULT_MAX_SIZE;

    if let Some(inline) = self.data_url {
      is_inline = inline;
    }

    tracing::trace!(
      "asset {:?} with size {}, is inlined {}",
      args.uri,
      size,
      is_inline
    );

    Ok(Box::new(AssetModule::new(module_type, is_inline, buf)))
  }
}

#[derive(Debug)]
struct AssetModule {
  module_type: ModuleType,
  inline: bool, // if the module is not inlined, then it will be regarded as a resource
  buf: Vec<u8>,
}

impl AssetModule {
  fn new(module_type: ModuleType, inline: bool, buf: Vec<u8>) -> Self {
    Self {
      module_type,
      inline,
      buf,
    }
  }
}

impl Module for AssetModule {
  #[inline(always)]
  fn module_type(&self) -> ModuleType {
    self.module_type
  }

  #[inline(always)]
  fn source_types(
    &self,
    _module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> HashSet<SourceType> {
    HashSet::from_iter(vec![SourceType::Asset, SourceType::JavaScript])
  }

  fn render(
    &self,
    requested_source_type: SourceType,
    module: &rspack_core::ModuleGraphModule,
    compilation: &rspack_core::Compilation,
  ) -> Result<Option<ModuleRenderResult>> {
    let result = match requested_source_type {
      SourceType::JavaScript => Some(ModuleRenderResult::JavaScript(format!(
        r#"rs.define("{}", function(__rspack_require__, module, exports) {{
  "use strict";
  module.exports = "{}";
}});
"#,
        module.id,
        if self.inline {
          format!(
            "data:{};base64,{}",
            MimeGuess::from_path(Path::new(&module.uri))
              .first()
              .ok_or_else(|| anyhow::format_err!("failed to guess mime type of {}", module.id))?,
            base64::encode(&self.buf)
          )
        } else {
          let path = Path::new(&module.id);
          format!(
            "{}{}",
            "/",
            compilation.options.output.asset_module_filename.filename(
              path
                .file_stem()
                .and_then(OsStr::to_str)
                .ok_or_else(|| anyhow::anyhow!("failed"))?
                .to_owned(),
              path
                .extension()
                .and_then(OsStr::to_str)
                .map(|str| format!("{}{}", ".", str))
                .ok_or_else(|| anyhow::anyhow!("failed"))?
            ),
          )
        }
      ))),
      SourceType::Asset => {
        if self.inline {
          None
        } else {
          Some(ModuleRenderResult::Asset(self.buf.clone()))
        }
      }
      _ => None,
    };

    Ok(result)
  }
}

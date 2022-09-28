use std::{ffi::OsStr, path::Path};

use async_trait::async_trait;
use rayon::prelude::*;
use rspack_error::{Error, IntoTWithDiagnosticArray, Result};

use rspack_core::{
  get_contenthash,
  rspack_sources::{RawSource, SourceExt},
  AssetParserDataUrlOption, AssetParserOptions, FilenameRenderOptions, GenerationResult,
  ParseContext, ParserAndGenerator, Plugin, PluginContext, PluginRenderManifestHookOutput,
  RenderManifestArgs, RenderManifestEntry, SourceType,
};

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

static ASSET_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::Asset, SourceType::JavaScript];

static ASSET_SOURCE_MODULE_SOURCE_TYPE_LIST: &[SourceType; 1] = &[SourceType::JavaScript];

#[derive(Debug)]
enum DataUrlOption {
  Inline(bool),
  Source,
  Auto(Option<AssetParserDataUrlOption>),
}

type IsInline = bool;

#[derive(Debug)]
enum CanonicalizedDataUrlOption {
  Source,
  Asset(IsInline),
}

impl CanonicalizedDataUrlOption {
  fn is_source(&self) -> bool {
    matches!(self, CanonicalizedDataUrlOption::Source)
  }

  fn is_inline(&self) -> bool {
    matches!(self, CanonicalizedDataUrlOption::Asset(true))
  }

  fn is_external(&self) -> bool {
    matches!(self, CanonicalizedDataUrlOption::Asset(false))
  }
}

#[derive(Debug)]
pub struct AssetParserAndGenerator {
  data_url: DataUrlOption,
  parsed_asset_config: Option<CanonicalizedDataUrlOption>,
}

impl AssetParserAndGenerator {
  pub fn with_auto(option: Option<AssetParserDataUrlOption>) -> Self {
    Self {
      data_url: DataUrlOption::Auto(option),
      parsed_asset_config: None,
    }
  }

  pub fn with_inline() -> Self {
    Self {
      data_url: DataUrlOption::Inline(true),
      parsed_asset_config: None,
    }
  }

  pub fn with_resource() -> Self {
    Self {
      data_url: DataUrlOption::Inline(false),
      parsed_asset_config: None,
    }
  }

  pub fn with_source() -> Self {
    Self {
      data_url: DataUrlOption::Source,
      parsed_asset_config: None,
    }
  }
}

// Webpack's default parser.dataUrlCondition.maxSize
const DEFAULT_MAX_SIZE: u32 = 8096;

impl ParserAndGenerator for AssetParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    if let Some(config) = self.parsed_asset_config.as_ref() {
      if config.is_source() || config.is_inline() {
        ASSET_SOURCE_MODULE_SOURCE_TYPE_LIST
      } else {
        ASSET_MODULE_SOURCE_TYPE_LIST
      }
    } else {
      panic!("Failed to read source types for asset module")
    }
  }

  fn parse(
    &mut self,
    parse_context: rspack_core::ParseContext,
  ) -> Result<rspack_error::TWithDiagnosticArray<rspack_core::ParseResult>> {
    let ParseContext { source, .. } = parse_context;

    let size = source.size();

    self.parsed_asset_config = match &self.data_url {
      DataUrlOption::Source => Some(CanonicalizedDataUrlOption::Source),
      DataUrlOption::Inline(val) => Some(CanonicalizedDataUrlOption::Asset(*val)),
      DataUrlOption::Auto(option) => {
        let limit_size = option
          .as_ref()
          .and_then(|x| x.max_size)
          .unwrap_or(DEFAULT_MAX_SIZE);
        Some(CanonicalizedDataUrlOption::Asset(
          size <= limit_size as usize,
        ))
      }
    };

    Ok(
      rspack_core::ParseResult {
        // Assets do not have dependencies
        dependencies: vec![],
        ast_or_source: source.into(),
      }
      .with_empty_diagnostic(),
    )
  }

  // Safety: `original_source` and `ast_and_source` are available in code generation.
  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    requested_source_type: SourceType,
    ast_or_source: &rspack_core::AstOrSource,
    mgm: &rspack_core::ModuleGraphModule,
    compilation: &rspack_core::Compilation,
  ) -> Result<rspack_core::GenerationResult> {
    let parsed_asset_config = self.parsed_asset_config.as_ref().unwrap();
    let result = match requested_source_type {
      SourceType::JavaScript => Ok(GenerationResult {
        ast_or_source: RawSource::from(format!(
          r#"module.exports = {};"#,
          if parsed_asset_config.is_inline() {
            format!(
              r#""data:{};base64,{}""#,
              mime_guess::MimeGuess::from_path(Path::new(&mgm.module.request()))
                .first()
                .ok_or_else(|| anyhow::format_err!(
                  "failed to guess mime type of {}",
                  mgm.module.request()
                ))?,
              base64::encode(
                &ast_or_source
                  .as_source()
                  .expect("Expected source for asset generator, please file an issue.")
                  .buffer()
              )
            )
          } else if parsed_asset_config.is_external() {
            let request = mgm.module.request();
            let path = Path::new(&request);

            let file_name =
              compilation
                .options
                .output
                .asset_module_filename
                .render(FilenameRenderOptions {
                  filename: Some(
                    path
                      .file_stem()
                      .and_then(OsStr::to_str)
                      .ok_or_else(|| anyhow::anyhow!("Failed to get filename for asset/resource"))?
                      .to_owned(),
                  ),
                  extension: Some(
                    path
                      .extension()
                      .and_then(OsStr::to_str)
                      .map(|str| format!("{}{}", ".", str))
                      .ok_or_else(|| {
                        anyhow::anyhow!("Failed to get extension for asset/resource")
                      })?,
                  ),
                  id: None,
                  contenthash: None,
                  chunkhash: None,
                  hash: None,
                });
            let public_path = compilation.options.output.public_path.public_path();
            format!(r#""{}{}""#, public_path, file_name)
          } else if parsed_asset_config.is_source() {
            format!(
              r"{:?}",
              ast_or_source
                .as_source()
                .expect("Expected source for asset generator, please file an issue.")
                .source()
            )
          } else {
            unreachable!()
          }
        ))
        .boxed()
        .into(),
      }),
      SourceType::Asset => {
        if parsed_asset_config.is_source() || parsed_asset_config.is_inline() {
          Err(Error::InternalError(
            "Inline or Source asset does not have source type `asset`".to_string(),
          ))
        } else {
          // Safety: This is safe because we returned the source in parser.
          Ok(GenerationResult {
            ast_or_source: RawSource::from(
              ast_or_source
                .as_source()
                .expect("Expected source for asset generator, please file an issue.")
                .buffer()
                .to_vec(),
            )
            .boxed()
            .into(),
          })
        }
      }
      t => Err(Error::InternalError(format!(
        "Unsupported source type {:?} for plugin JavaScript",
        t
      ))),
    };

    result
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
    let data_url_condition = self
      .config
      .parse_options
      .as_ref()
      .and_then(|x| x.data_url_condition.clone());

    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::Asset,
      Box::new(move || {
        Box::new(AssetParserAndGenerator::with_auto(
          data_url_condition.clone(),
        ))
      }),
    );

    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::AssetInline,
      Box::new(|| Box::new(AssetParserAndGenerator::with_inline())),
    );

    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::AssetResource,
      Box::new(|| Box::new(AssetParserAndGenerator::with_resource())),
    );

    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::AssetSource,
      Box::new(|| Box::new(AssetParserAndGenerator::with_source())),
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
        // TODO: this logic is definitely not performant, move to compilation afterwards
        Ok(
          module
            .module
            .code_generation(module, compilation)
            .map(|source| {
              source
                .inner()
                .get(&SourceType::Asset)
                .map(|source| source.ast_or_source.clone().try_into_source().unwrap())
                .map(|asset| {
                  let contenthash = Some(get_contenthash(&asset).to_string());
                  let chunkhash = None;
                  // Some(get_chunkhash(compilation, &args.chunk_ukey, module_graph).to_string());
                  // let hash = Some(get_hash(compilation).to_string());
                  let hash = None;

                  let path = Path::new(&module.id);
                  Some(RenderManifestEntry::new(
                    asset,
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
                        contenthash,
                        chunkhash,
                        hash,
                      }),
                  ))
                })
            })
            .unwrap() // TODO: remove this unwrap
            .flatten(),
        )
      })
      .collect::<Result<Vec<Option<RenderManifestEntry>>>>()?
      .into_par_iter()
      .flatten()
      .collect::<Vec<RenderManifestEntry>>();

    Ok(assets)
  }
}

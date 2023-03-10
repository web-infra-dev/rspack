use std::{
  collections::hash_map::DefaultHasher,
  ffi::OsStr,
  hash::BuildHasherDefault,
  hash::{Hash, Hasher},
  path::Path,
  sync::Arc,
};

use async_trait::async_trait;
use dashmap::DashMap;
use rayon::prelude::*;
use rspack_core::{
  get_contenthash,
  rspack_sources::{RawSource, SourceExt},
  runtime_globals, AssetParserDataUrlOption, AssetParserOptions, AstOrSource,
  FilenameRenderOptions, GenerateContext, GenerationResult, Module, ModuleIdentifier, ParseContext,
  ParserAndGenerator, PathData, Plugin, PluginContext, PluginRenderManifestHookOutput,
  RenderManifestArgs, RenderManifestEntry, SourceType,
};
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result};
use rspack_identifier::IdentifierHasher;
use sugar_path::SugarPath;

#[derive(Debug)]
pub struct AssetConfig {
  pub parse_options: Option<AssetParserOptions>,
}

pub type ModuleIdToFileName =
  Arc<DashMap<ModuleIdentifier, String, BuildHasherDefault<IdentifierHasher>>>;

#[derive(Debug)]
pub struct AssetPlugin {
  config: AssetConfig,
  module_id_to_filename_without_ext: ModuleIdToFileName,
}

impl AssetPlugin {
  pub fn new(config: AssetConfig) -> AssetPlugin {
    AssetPlugin {
      config,
      module_id_to_filename_without_ext: Default::default(),
    }
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

const ASSET_INLINE: bool = true;
const ASSET_EXTERNAL: bool = false;

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
    matches!(self, CanonicalizedDataUrlOption::Asset(ASSET_INLINE))
  }

  fn is_external(&self) -> bool {
    matches!(self, CanonicalizedDataUrlOption::Asset(ASSET_EXTERNAL))
  }
}

#[derive(Debug)]
pub struct AssetParserAndGenerator {
  data_url: DataUrlOption,
  parsed_asset_config: Option<CanonicalizedDataUrlOption>,
  module_id_to_filename: ModuleIdToFileName,
}

impl AssetParserAndGenerator {
  pub fn with_auto(option: Option<AssetParserDataUrlOption>) -> Self {
    Self {
      data_url: DataUrlOption::Auto(option),
      parsed_asset_config: None,
      module_id_to_filename: Default::default(),
    }
  }

  pub fn with_inline() -> Self {
    Self {
      data_url: DataUrlOption::Inline(true),
      parsed_asset_config: None,
      module_id_to_filename: Default::default(),
    }
  }

  pub fn with_resource() -> Self {
    Self {
      data_url: DataUrlOption::Inline(false),
      parsed_asset_config: None,
      module_id_to_filename: Default::default(),
    }
  }

  pub fn with_source() -> Self {
    Self {
      data_url: DataUrlOption::Source,
      parsed_asset_config: None,
      module_id_to_filename: Default::default(),
    }
  }

  pub(crate) fn into_with_module_id_to_filename_without_ext(
    mut self,
    module_id_to_filename_without_ext: ModuleIdToFileName,
  ) -> Self {
    self.module_id_to_filename = module_id_to_filename_without_ext;
    self
  }

  fn hash_for_ast_or_source(&self, ast_or_source: &AstOrSource) -> u64 {
    let mut hasher = DefaultHasher::new();
    ast_or_source.hash(&mut hasher);
    hasher.finish()
  }

  fn generate_external_content(
    &self,
    generate_context: &mut GenerateContext,
    filename: String,
  ) -> Result<String> {
    generate_context
      .runtime_requirements
      .insert(runtime_globals::PUBLIC_PATH);
    Ok(format!(
      r#"{} + "{}""#,
      runtime_globals::PUBLIC_PATH,
      filename
    ))
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
      // If module is failed to build, then the `parsed_asset_config` is not set.
      // Align with webpacks's asset module: https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/asset/AssetGenerator.js#L386
      ASSET_MODULE_SOURCE_TYPE_LIST
    }
  }

  fn size(&self, module: &dyn Module, source_type: &SourceType) -> f64 {
    let original_source_size = module.original_source().map_or(0, |source| source.size()) as f64;
    match source_type {
      SourceType::Asset => original_source_size,
      SourceType::JavaScript => {
        if module.original_source().is_none() {
          return 0.0;
        }

        let parsed_size = self.parsed_asset_config.as_ref().map(|config| {
          match config {
            CanonicalizedDataUrlOption::Source => original_source_size,
            CanonicalizedDataUrlOption::Asset(meta) => {
              match *meta {
                ASSET_INLINE => {
                  // copied from webpack's AssetGenerator
                  // roughly for data url
                  // Example: m.exports="data:image/png;base64,ag82/f+2=="
                  // 4/3 = base64 encoding
                  // 34 = ~ data url header + footer + rounding
                  original_source_size * 1.34 + 36.0
                }
                ASSET_EXTERNAL => {
                  // copied from webpack's AssetGenerator
                  // roughly for url
                  // Example: m.exports=r.p+"0123456789012345678901.ext"
                  42.0
                }
              }
            }
          }
        });

        parsed_size.unwrap_or_default()
      }
      _ => unreachable!(),
    }
  }

  fn parse(
    &mut self,
    parse_context: rspack_core::ParseContext,
  ) -> Result<rspack_error::TWithDiagnosticArray<rspack_core::ParseResult>> {
    let ParseContext {
      source, build_info, ..
    } = parse_context;
    build_info.strict = true;
    let size = source.size();

    self.parsed_asset_config = match &self.data_url {
      DataUrlOption::Source => Some(CanonicalizedDataUrlOption::Source),
      DataUrlOption::Inline(val) => Some(CanonicalizedDataUrlOption::Asset(*val)),
      DataUrlOption::Auto(option) => {
        let limit_size = parse_context
          .module_parser_options
          .and_then(|x| x.data_url_condition.as_ref().and_then(|d| d.max_size))
          .or(option.as_ref().and_then(|x| x.max_size))
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
        presentational_dependencies: vec![],
      }
      .with_empty_diagnostic(),
    )
  }

  // Safety: `original_source` and `ast_and_source` are available in code generation.
  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    ast_or_source: &rspack_core::AstOrSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<rspack_core::GenerationResult> {
    let parsed_asset_config = self.parsed_asset_config.as_ref().expect("TODO:");

    // Use [Rule.generator.filename] if it is set, otherwise use [output.assetModuleFilename].
    let asset_filename_template = generate_context
      .module_generator_options
      .and_then(|o| o.filename.as_ref())
      .unwrap_or(
        &generate_context
          .compilation
          .options
          .output
          .asset_module_filename,
      );
    let hash = hash_value_to_string(self.hash_for_ast_or_source(ast_or_source));

    let asset_filename = asset_filename_template.render(FilenameRenderOptions {
      name: module.as_normal_module().and_then(|m| {
        let p = Path::new(&m.resource_resolved_data().resource_path);
        p.file_stem().map(|s| s.to_string_lossy().to_string())
      }),
      path: module.as_normal_module().map(|m| {
        Path::new(&m.resource_resolved_data().resource_path)
          .relative(&generate_context.compilation.options.context)
          .to_string_lossy()
          .to_string()
      }),
      extension: module.as_normal_module().and_then(|m| {
        Path::new(&m.resource_resolved_data().resource_path)
          .extension()
          .and_then(OsStr::to_str)
          .map(|str| format!("{}{}", ".", str))
      }),
      id: None,
      contenthash: Some(hash.clone()),
      chunkhash: Some(hash.clone()),
      hash: Some(hash),
      query: Some("".to_string()),
    });

    self
      .module_id_to_filename
      .insert(module.identifier(), asset_filename.clone());

    let result = match generate_context.requested_source_type {
      SourceType::JavaScript => {
        let module = module.try_as_normal_module()?;
        let resource_path = &module.resource_resolved_data().resource_path;

        let exported_content = if parsed_asset_config.is_inline() {
          let encoded_source = format!(
            r#"data:{};base64,{}"#,
            mime_guess::MimeGuess::from_path(Path::new(resource_path))
              .first()
              .ok_or_else(|| anyhow::format_err!(
                "failed to guess mime type of {}",
                resource_path.display()
              ))?,
            rspack_base64::encode_to_string(
              ast_or_source
                .as_source()
                .expect("Expected source for asset generator, please file an issue.")
                .buffer()
            )
          );

          generate_context
            .data
            .insert("url".to_string(), encoded_source.clone());

          format!("\"{encoded_source}\"")
        } else {
          generate_context
            .data
            .insert("filename".to_string(), asset_filename.clone());

          if parsed_asset_config.is_external() {
            self.generate_external_content(generate_context, asset_filename)?
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
        };

        Ok(GenerationResult {
          ast_or_source: RawSource::from(format!(r#"module.exports = {exported_content};"#))
            .boxed()
            .into(),
        })
      }
      SourceType::Asset => {
        if parsed_asset_config.is_source() || parsed_asset_config.is_inline() {
          Err(internal_error!(
            "Inline or Source asset does not have source type `asset`"
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
      t => Err(internal_error!(format!(
        "Unsupported source type {t:?} for plugin JavaScript"
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

    {
      let module_id_to_filename_without_ext = self.module_id_to_filename_without_ext.clone();
      ctx.context.register_parser_and_generator_builder(
        rspack_core::ModuleType::Asset,
        Box::new(move || {
          Box::new(
            AssetParserAndGenerator::with_auto(data_url_condition.clone())
              .into_with_module_id_to_filename_without_ext(
                module_id_to_filename_without_ext.clone(),
              ),
          )
        }),
      );
    }

    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::AssetInline,
      Box::new(|| Box::new(AssetParserAndGenerator::with_inline())),
    );

    {
      let module_id_to_filename_without_ext = self.module_id_to_filename_without_ext.clone();
      ctx.context.register_parser_and_generator_builder(
        rspack_core::ModuleType::AssetResource,
        Box::new(move || {
          Box::new(
            AssetParserAndGenerator::with_resource().into_with_module_id_to_filename_without_ext(
              module_id_to_filename_without_ext.clone(),
            ),
          )
        }),
      );
    }
    {
      let module_id_to_filename_without_ext = self.module_id_to_filename_without_ext.clone();
      ctx.context.register_parser_and_generator_builder(
        rspack_core::ModuleType::AssetSource,
        Box::new(move || {
          Box::new(
            AssetParserAndGenerator::with_source().into_with_module_id_to_filename_without_ext(
              module_id_to_filename_without_ext.clone(),
            ),
          )
        }),
      );
    }

    Ok(())
  }

  async fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: RenderManifestArgs<'_>,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let chunk = args.chunk();
    let module_graph = &compilation.module_graph;

    let ordered_modules = compilation
      .chunk_graph
      .get_chunk_modules(&args.chunk_ukey, module_graph);

    let assets = ordered_modules
      .par_iter()
      .filter(|m| {
        let module = compilation
          .module_graph
          .module_by_identifier(&m.identifier())
          .ok_or_else(|| internal_error!("Failed to get module".to_owned()))
          // FIXME: use result
          .expect("Failed to get module");
        module.source_types().contains(&SourceType::Asset)
      })
      .map(|m| {
        let code_gen_result = compilation
          .code_generation_results
          .get(&m.identifier(), Some(&chunk.runtime))?;

        let module = compilation
          .module_graph
          .module_by_identifier(&m.identifier())
          .ok_or_else(|| internal_error!("Failed to get module".to_owned()))
          .expect("Failed to get module")
          .as_normal_module()
          .expect("Normal module expected");

        let result = code_gen_result
          .get(&SourceType::Asset)
          .map(|result| result.ast_or_source.clone().try_into_source())
          .transpose()?
          .map(|source| {
            let asset_filename = self
              .module_id_to_filename_without_ext
              .get(&m.identifier())
              .map(|s| s.clone());
            let contenthash = Some(get_contenthash(&source).to_string());
            let chunkhash = None;

            let hash = None;

            let path = Path::new(&module.resource_resolved_data().resource_path)
              .relative(&compilation.options.context);
            RenderManifestEntry::new(
              source,
              asset_filename.unwrap_or_else(|| {
                args
                  .compilation
                  .options
                  .output
                  .asset_module_filename
                  .render(FilenameRenderOptions {
                    name: Some(
                      path
                        .file_stem()
                        .map(|s| s.to_string_lossy())
                        .expect("TODO:")
                        .to_string(),
                    ),
                    path: path.parent().map(|p| p.to_string_lossy().to_string() + "/"),
                    extension: path
                      .extension()
                      .and_then(OsStr::to_str)
                      .map(|str| format!("{}{}", ".", str)),
                    id: None,
                    contenthash,
                    chunkhash,
                    hash,
                    query: Some("".to_string()),
                  })
              }),
              PathData {
                chunk_ukey: args.chunk_ukey,
              },
            )
          });

        Ok(result)
      })
      .collect::<Result<Vec<Option<RenderManifestEntry>>>>()?
      .into_par_iter()
      .flatten()
      .collect::<Vec<RenderManifestEntry>>();

    Ok(assets)
  }
}

fn hash_value_to_string(hash: u64) -> String {
  format!("{hash:x}")
}

#![feature(let_chains)]

use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

use async_trait::async_trait;
use rayon::prelude::*;
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  AssetParserDataUrlOption, AssetParserOptions, AstOrSource, BuildMetaDefaultObject,
  BuildMetaExportsType, CodeGenerationDataAssetInfo, CodeGenerationDataFilename,
  CodeGenerationDataUrl, Compilation, GenerateContext, GenerationResult, Module, NormalModule,
  ParseContext, ParserAndGenerator, PathData, Plugin, PluginContext,
  PluginRenderManifestHookOutput, RenderManifestArgs, RenderManifestEntry, ResourceData,
  RuntimeGlobals, SourceType,
};
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result};
use rspack_util::identifier::make_paths_relative;

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

const DEFAULT_ENCODING: &str = "base64";

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
      .insert(RuntimeGlobals::PUBLIC_PATH);
    Ok(format!(
      r#"{} + "{}""#,
      RuntimeGlobals::PUBLIC_PATH,
      filename
    ))
  }

  fn get_mimetype(&self, resource_data: &ResourceData) -> Result<String> {
    if let Some(mimetype) = &resource_data.mimetype && let Some(parameters) = &resource_data.parameters {
      return Ok(format!("{mimetype}{parameters}"));
    }
    mime_guess::MimeGuess::from_path(&resource_data.resource_path)
      .first_raw()
      .map(ToOwned::to_owned)
      .ok_or_else(|| {
        internal_error!(
          "failed to guess mime type of {:?}",
          resource_data.resource_path
        )
      })
  }

  fn get_encoding(&self, resource_data: &ResourceData) -> String {
    if let Some(encoding) = &resource_data.encoding {
      return encoding.to_owned();
    }
    String::from(DEFAULT_ENCODING)
  }

  fn get_encoded_content(
    &self,
    resource_data: &ResourceData,
    encoding: &str,
    source: &BoxSource,
  ) -> Result<String> {
    if let Some(encoded_content) = &resource_data.encoded_content {
      return Ok(encoded_content.to_owned());
    }
    if encoding.is_empty() {
      // to_lossy_string
      return Ok(urlencoding::encode_binary(&source.buffer()).into_owned());
    }
    if encoding == DEFAULT_ENCODING {
      return Ok(rspack_base64::encode_to_string(source.buffer()));
    }
    Err(internal_error!("Unsupported encoding {encoding}"))
  }

  fn get_source_file_name(&self, module: &NormalModule, compilation: &Compilation) -> String {
    let relative = make_paths_relative(
      &compilation.options.context.to_string_lossy(),
      &module
        .match_resource()
        .unwrap_or(module.resource_resolved_data())
        .resource,
    );
    if let Some(stripped) = relative.strip_prefix("./") {
      return stripped.to_owned();
    }
    relative
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
      source,
      build_meta,
      build_info,
      ..
    } = parse_context;
    build_info.strict = true;
    build_meta.exports_type = BuildMetaExportsType::Default;
    build_meta.default_object = BuildMetaDefaultObject::False;
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
        code_replace_source_dependencies: vec![],
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
    let contenthash = hash_value_to_string(self.hash_for_ast_or_source(ast_or_source));
    let normal_module = module
      .as_normal_module()
      .expect("module should be a NormalModule in AssetParserAndGenerator");
    let source_file_name = self.get_source_file_name(normal_module, generate_context.compilation);
    let (asset_filename, asset_info) = generate_context.compilation.get_asset_path_with_info(
      asset_filename_template,
      PathData::default()
        .module(module)
        .chunk_graph(&generate_context.compilation.chunk_graph)
        .content_hash(&contenthash)
        .hash(&contenthash)
        .filename(&source_file_name),
    );

    let result = match generate_context.requested_source_type {
      SourceType::JavaScript => {
        let module = module.try_as_normal_module()?;

        let exported_content = if parsed_asset_config.is_inline() {
          let resource_data = module.resource_resolved_data();
          let mimetype = self.get_mimetype(resource_data)?;
          let encoding = self.get_encoding(resource_data);
          let encoded_content = self.get_encoded_content(
            resource_data,
            &encoding,
            ast_or_source
              .as_source()
              .expect("Expected source for asset generator, please file an issue."),
          )?;
          let encoded_source = format!(
            r#"data:{mimetype}{},{encoded_content}"#,
            if encoding.is_empty() {
              String::new()
            } else {
              format!(";{encoding}")
            }
          );

          generate_context
            .data
            .insert(CodeGenerationDataUrl::new(encoded_source.clone()));

          serde_json::to_string(&encoded_source).map_err(|e| internal_error!(e.to_string()))?
        } else {
          generate_context
            .data
            .insert(CodeGenerationDataFilename::new(asset_filename.clone()));
          generate_context
            .data
            .insert(CodeGenerationDataAssetInfo::new(asset_info));

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
      Box::new(move || Box::new(AssetParserAndGenerator::with_resource())),
    );
    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::AssetSource,
      Box::new(move || Box::new(AssetParserAndGenerator::with_source())),
    );

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
          && compilation
            .include_module_ids
            .contains(&module.identifier())
      })
      .map(|m| {
        let code_gen_result = compilation
          .code_generation_results
          .get(&m.identifier(), Some(&chunk.runtime))?;

        let result = code_gen_result
          .get(&SourceType::Asset)
          .map(|result| result.ast_or_source.clone().try_into_source())
          .transpose()?
          .map(|source| {
            let asset_filename = code_gen_result
              .data
              .get::<CodeGenerationDataFilename>()
              .expect("should have filename for asset module")
              .inner();
            let asset_info = code_gen_result
              .data
              .get::<CodeGenerationDataAssetInfo>()
              .expect("should have asset_info")
              .inner();
            RenderManifestEntry::new(source, asset_filename.to_owned(), asset_info.to_owned())
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

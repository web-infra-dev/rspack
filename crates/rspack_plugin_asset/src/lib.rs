#![feature(let_chains)]

use std::{collections::HashMap, hash::Hash};

use async_trait::async_trait;
use rayon::prelude::*;
use rkyv::{from_bytes, to_bytes, AlignedVec, Archive, Deserialize, Serialize};
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  tree_shaking::{
    analyzer::OptimizeAnalyzer, asset_module::AssetModule, visitor::OptimizeAnalyzeResult,
  },
  AssetGeneratorDataUrl, AssetGeneratorDataUrlFnArgs, AssetParserDataUrl, BuildExtraDataType,
  BuildMetaDefaultObject, BuildMetaExportsType, CodeGenerationDataAssetInfo,
  CodeGenerationDataFilename, CodeGenerationDataUrl, Compilation, CompilerOptions, GenerateContext,
  Module, ModuleType, NormalModule, ParseContext, ParserAndGenerator, PathData, Plugin,
  PluginContext, PluginRenderManifestHookOutput, RenderManifestArgs, RenderManifestEntry,
  ResourceData, RuntimeGlobals, SourceType, NAMESPACE_OBJECT_EXPORT,
};
use rspack_error::{error, IntoTWithDiagnosticArray, Result};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::identifier::make_paths_relative;

#[derive(Debug)]
pub struct AssetPlugin;

static ASSET_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::Asset, SourceType::JavaScript];

static ASSET_SOURCE_MODULE_SOURCE_TYPE_LIST: &[SourceType; 1] = &[SourceType::JavaScript];

const DEFAULT_ENCODING: &str = "base64";

#[derive(Debug)]
enum DataUrlOptions {
  Inline(bool),
  Source,
  Auto(Option<AssetParserDataUrl>),
}

type IsInline = bool;

const ASSET_INLINE: bool = true;
const ASSET_RESOURCE: bool = false;

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq), check_bytes)]
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

  fn is_resource(&self) -> bool {
    matches!(self, CanonicalizedDataUrlOption::Asset(ASSET_RESOURCE))
  }
}

#[derive(Debug)]
pub struct AssetParserAndGenerator {
  data_url: DataUrlOptions,
  parsed_asset_config: Option<CanonicalizedDataUrlOption>,
}

impl AssetParserAndGenerator {
  pub fn with_auto(option: Option<AssetParserDataUrl>) -> Self {
    Self {
      data_url: DataUrlOptions::Auto(option),
      parsed_asset_config: None,
    }
  }

  pub fn with_inline() -> Self {
    Self {
      data_url: DataUrlOptions::Inline(true),
      parsed_asset_config: None,
    }
  }

  pub fn with_resource() -> Self {
    Self {
      data_url: DataUrlOptions::Inline(false),
      parsed_asset_config: None,
    }
  }

  pub fn with_source() -> Self {
    Self {
      data_url: DataUrlOptions::Source,
      parsed_asset_config: None,
    }
  }

  fn hash_for_source(
    &self,
    source: &BoxSource,
    compiler_options: &CompilerOptions,
  ) -> RspackHashDigest {
    let mut hasher = RspackHash::from(&compiler_options.output);
    source.hash(&mut hasher);
    hasher.digest(&compiler_options.output.hash_digest)
  }

  fn get_data_url(
    &self,
    resource_data: &ResourceData,
    data_url: Option<&AssetGeneratorDataUrl>,
    source: &BoxSource,
  ) -> Option<String> {
    let func_args = AssetGeneratorDataUrlFnArgs {
      filename: resource_data.resource_path.to_string_lossy().to_string(),
      content: source.source().into_owned().to_string(),
    };

    if let Some(AssetGeneratorDataUrl::Func(data_url)) = data_url {
      return Some(data_url(func_args).expect("call data_url function failed"));
    }
    None
  }

  fn get_mimetype(
    &self,
    resource_data: &ResourceData,
    data_url: Option<&AssetGeneratorDataUrl>,
  ) -> Result<String> {
    if let Some(AssetGeneratorDataUrl::Options(data_url)) = data_url
      && let Some(mimetype) = &data_url.mimetype
    {
      return Ok(mimetype.to_owned());
    }
    if let Some(mimetype) = &resource_data.mimetype
      && let Some(parameters) = &resource_data.parameters
    {
      return Ok(format!("{mimetype}{parameters}"));
    }
    mime_guess::MimeGuess::from_path(&resource_data.resource_path)
      .first_raw()
      .map(ToOwned::to_owned)
      .ok_or_else(|| {
        error!(
          "failed to guess mime type of {:?}",
          resource_data.resource_path
        )
      })
  }

  fn get_encoding(
    &self,
    resource_data: &ResourceData,
    data_url: Option<&AssetGeneratorDataUrl>,
  ) -> String {
    if let Some(AssetGeneratorDataUrl::Options(data_url)) = data_url
      && let Some(encoding) = &data_url.encoding
    {
      return encoding.to_string();
    }
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
    Err(error!("Unsupported encoding {encoding}"))
  }

  fn get_source_file_name(&self, module: &NormalModule, compilation: &Compilation) -> String {
    let relative = make_paths_relative(
      compilation.options.context.as_ref(),
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
                ASSET_RESOURCE => {
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
      module_type,
      compiler_options,
      module_identifier,
      ..
    } = parse_context;
    build_info.strict = true;
    build_meta.exports_type = BuildMetaExportsType::Default;
    build_meta.default_object = BuildMetaDefaultObject::False;
    let size = source.size();

    self.parsed_asset_config = match &self.data_url {
      DataUrlOptions::Source => Some(CanonicalizedDataUrlOption::Source),
      DataUrlOptions::Inline(val) => Some(CanonicalizedDataUrlOption::Asset(*val)),
      DataUrlOptions::Auto(option) => {
        let limit_size = parse_context
          .module_parser_options
          .and_then(|x| {
            x.get_asset(module_type)
              .and_then(|x| x.data_url_condition.as_ref())
              .and_then(|x| match x {
                AssetParserDataUrl::Options(x) => x.max_size,
              })
          })
          .or_else(|| {
            option.as_ref().and_then(|x| match x {
              AssetParserDataUrl::Options(x) => x.max_size,
            })
          })
          .unwrap_or(DEFAULT_MAX_SIZE);
        Some(CanonicalizedDataUrlOption::Asset(
          size <= limit_size as usize,
        ))
      }
    };
    let analyze_result = if compiler_options.builtins.tree_shaking.enable() {
      AssetModule::new(module_identifier).analyze()
    } else {
      OptimizeAnalyzeResult::default()
    };

    Ok(
      rspack_core::ParseResult {
        // Assets do not have dependencies
        dependencies: vec![],
        blocks: vec![],
        source,
        presentational_dependencies: vec![],
        analyze_result,
        side_effects_bailout: None,
      }
      .with_empty_diagnostic(),
    )
  }

  // Safety: `original_source` and `ast_and_source` are available in code generation.
  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    source: &BoxSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    let compilation = generate_context.compilation;
    let module_type = module.module_type();
    let parsed_asset_config = self
      .parsed_asset_config
      .as_ref()
      .expect("should have parsed_asset_config in generate phase");
    let normal_module = module
      .as_normal_module()
      .expect("module should be a NormalModule in AssetParserAndGenerator");

    let result = match generate_context.requested_source_type {
      SourceType::JavaScript => {
        let exported_content = if parsed_asset_config.is_inline() {
          let resource_data: &ResourceData = normal_module.resource_resolved_data();
          let data_url = generate_context
            .module_generator_options
            .and_then(|x| x.asset_data_url(module_type));

          let encoded_source: String;

          if let Some(custom_data_url) = self.get_data_url(resource_data, data_url, source) {
            encoded_source = custom_data_url;
          } else {
            let mimetype = self.get_mimetype(resource_data, data_url)?;
            let encoding = self.get_encoding(resource_data, data_url);
            let encoded_content = self.get_encoded_content(resource_data, &encoding, source)?;
            encoded_source = format!(
              r#"data:{mimetype}{},{encoded_content}"#,
              if encoding.is_empty() {
                String::new()
              } else {
                format!(";{encoding}")
              }
            );
          }

          generate_context
            .data
            .insert(CodeGenerationDataUrl::new(encoded_source.clone()));

          serde_json::to_string(&encoded_source).map_err(|e| error!(e.to_string()))?
        } else if parsed_asset_config.is_resource() {
          // Use [Rule.generator.filename] if it is set, otherwise use [output.assetModuleFilename].
          let asset_filename_template = generate_context
            .module_generator_options
            .and_then(|x| x.asset_filename(module_type))
            .unwrap_or(&compilation.options.output.asset_module_filename);

          let contenthash = self.hash_for_source(source, &compilation.options);
          let contenthash = contenthash.rendered(compilation.options.output.hash_digest_length);

          let source_file_name = self.get_source_file_name(normal_module, compilation);
          let (filename, mut asset_info) = compilation.get_asset_path_with_info(
            asset_filename_template,
            PathData::default()
              .module(module)
              .chunk_graph(&generate_context.compilation.chunk_graph)
              .content_hash(contenthash)
              .hash(contenthash)
              .filename(&source_file_name),
          );

          let asset_path = if let Some(public_path) = generate_context
            .module_generator_options
            .and_then(|x| x.asset_public_path(module_type))
          {
            let public_path = public_path.render(compilation, &filename);
            serde_json::to_string(&format!("{public_path}{filename}"))
              .map_err(|e| error!(e.to_string()))?
          } else {
            generate_context
              .runtime_requirements
              .insert(RuntimeGlobals::PUBLIC_PATH);
            format!(r#"{} + "{}""#, RuntimeGlobals::PUBLIC_PATH, filename)
          };
          asset_info.set_source_filename(source_file_name);

          generate_context
            .data
            .insert(CodeGenerationDataFilename::new(
              filename,
              generate_context
                .module_generator_options
                .and_then(|x| x.asset_public_path(module_type))
                .unwrap_or_else(|| &compilation.options.output.public_path)
                .clone(),
            ));
          generate_context
            .data
            .insert(CodeGenerationDataAssetInfo::new(asset_info));

          asset_path
        } else if parsed_asset_config.is_source() {
          format!(r"{:?}", source.source())
        } else {
          unreachable!()
        };
        if let Some(ref mut scope) = generate_context.concatenation_scope {
          scope.register_namespace_export(NAMESPACE_OBJECT_EXPORT);
          // TODO: inspect supportConst https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/asset/AssetGenerator.js#L382-L386
          Ok(
            RawSource::from(format!(
              r#"const {NAMESPACE_OBJECT_EXPORT} = {exported_content};"#
            ))
            .boxed(),
          )
        } else {
          generate_context
            .runtime_requirements
            .insert(RuntimeGlobals::MODULE);

          Ok(RawSource::from(format!(r#"module.exports = {exported_content};"#)).boxed())
        }
      }
      SourceType::Asset => {
        if parsed_asset_config.is_source() || parsed_asset_config.is_inline() {
          Err(error!(
            "Inline or Source asset does not have source type `asset`"
          ))
        } else {
          Ok(RawSource::from(source.buffer().to_vec()).boxed())
        }
      }
      _ => panic!(
        "Unsupported source type: {:?}",
        generate_context.requested_source_type
      ),
    };

    result
  }
  fn store(&self, extra_data: &mut HashMap<BuildExtraDataType, AlignedVec>) {
    extra_data.insert(
      BuildExtraDataType::AssetParserAndGenerator,
      to_bytes::<_, 256>(&self.parsed_asset_config).expect("Failed to store extra data"),
    );
  }

  fn resume(&mut self, extra_data: &HashMap<BuildExtraDataType, AlignedVec>) {
    if let Some(data) = extra_data.get(&BuildExtraDataType::AssetParserAndGenerator) {
      self.parsed_asset_config = from_bytes::<Option<CanonicalizedDataUrlOption>>(data)
        .expect("Failed to resume extra data");
    }
  }
}

#[async_trait]
impl Plugin for AssetPlugin {
  fn name(&self) -> &'static str {
    "asset"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    let data_url_condition = options
      .module
      .parser
      .as_ref()
      .and_then(|x| x.get(&ModuleType::Asset))
      .and_then(|x| x.get_asset(&ModuleType::Asset))
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
    let module_graph = &compilation.get_module_graph();

    let ordered_modules = compilation
      .chunk_graph
      .get_chunk_modules(&args.chunk_ukey, module_graph);

    let assets = ordered_modules
      .par_iter()
      .filter(|m| {
        let module = compilation
          .get_module_graph()
          .module_by_identifier(&m.identifier())
          // FIXME: use result
          .expect("Failed to get module");

        let all_incoming_analyzed = module_graph
          .get_incoming_connections(module)
          .iter()
          .all(|c| {
            if let Some(original_module_identifier) = c.original_module_identifier {
              module_graph
                .module_graph_module_by_identifier(&original_module_identifier)
                .map(|original_module| {
                  !compilation
                    .bailout_module_identifiers
                    .contains_key(&original_module.module_identifier)
                    && original_module.module_type.is_js_like()
                })
                .unwrap_or(false)
            } else {
              false
            }
          });

        module.source_types().contains(&SourceType::Asset)
          && (!all_incoming_analyzed
            || compilation
              .include_module_ids
              .contains(&module.identifier()))
      })
      .map(|m| {
        let code_gen_result = compilation
          .code_generation_results
          .get(&m.identifier(), Some(&chunk.runtime));

        let result = code_gen_result.get(&SourceType::Asset).map(|source| {
          let asset_filename = code_gen_result
            .data
            .get::<CodeGenerationDataFilename>()
            .expect("should have filename for asset module")
            .filename();
          let asset_info = code_gen_result
            .data
            .get::<CodeGenerationDataAssetInfo>()
            .expect("should have asset_info")
            .inner();
          RenderManifestEntry::new(
            source.clone(),
            asset_filename.to_owned(),
            asset_info.to_owned(),
            true,
            true,
          )
        });

        Ok(result)
      })
      .collect::<Result<Vec<Option<RenderManifestEntry>>>>()?
      .into_par_iter()
      .flatten()
      .collect::<Vec<RenderManifestEntry>>();

    Ok(assets.with_empty_diagnostic())
  }
}

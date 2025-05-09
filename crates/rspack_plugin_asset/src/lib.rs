#![feature(let_chains)]

use std::{borrow::Cow, hash::Hasher, path::PathBuf};

use asset_exports_dependency::AssetExportsDependency;
use async_trait::async_trait;
use rayon::prelude::*;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  AssetGeneratorDataUrl, AssetGeneratorDataUrlFnCtx, AssetGeneratorImportMode, AssetInfo,
  AssetParserDataUrl, BuildMetaDefaultObject, BuildMetaExportsType, ChunkGraph, ChunkUkey,
  CodeGenerationDataAssetInfo, CodeGenerationDataFilename, CodeGenerationDataUrl,
  CodeGenerationPublicPathAutoReplace, Compilation, CompilationRenderManifest, CompilerOptions,
  Filename, GenerateContext, GeneratorOptions, Module, ModuleGraph, NormalModule, ParseContext,
  ParserAndGenerator, PathData, Plugin, PublicPath, RenderManifestEntry, ResourceData,
  RuntimeGlobals, RuntimeSpec, SourceType, NAMESPACE_OBJECT_EXPORT,
};
use rspack_error::{
  error, Diagnostic, IntoTWithDiagnosticArray, Result, ToStringResultToRspackResultExt,
};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::{ext::DynHash, identifier::make_paths_relative};

mod asset_exports_dependency;

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_ASSET_AUTO_PUBLIC_PATH__";

#[plugin]
#[derive(Debug, Default)]
pub struct AssetPlugin;

static ASSET_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::Asset, SourceType::JavaScript];

static ASSET_SOURCE_MODULE_SOURCE_TYPE_LIST: &[SourceType; 1] = &[SourceType::JavaScript];

const DEFAULT_ENCODING: &str = "base64";

#[cacheable]
#[derive(Debug)]
enum DataUrlOptions {
  Inline(bool),
  Source,
  Auto(Option<AssetParserDataUrl>),
}

type IsInline = bool;

const ASSET_INLINE: bool = true;
const ASSET_RESOURCE: bool = false;

#[cacheable]
#[derive(Debug, Clone)]
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

#[cacheable]
#[derive(Debug)]
pub struct AssetParserAndGenerator {
  emit: bool,
  data_url: DataUrlOptions,
  parsed_asset_config: Option<CanonicalizedDataUrlOption>,
}

impl AssetParserAndGenerator {
  pub fn with_auto(option: Option<AssetParserDataUrl>, emit: bool) -> Self {
    Self {
      emit,
      data_url: DataUrlOptions::Auto(option),
      parsed_asset_config: None,
    }
  }

  pub fn with_inline() -> Self {
    Self {
      emit: false,
      data_url: DataUrlOptions::Inline(true),
      parsed_asset_config: None,
    }
  }

  pub fn with_resource(emit: bool) -> Self {
    Self {
      emit,
      data_url: DataUrlOptions::Inline(false),
      parsed_asset_config: None,
    }
  }

  pub fn with_source() -> Self {
    Self {
      emit: false,
      data_url: DataUrlOptions::Source,
      parsed_asset_config: None,
    }
  }

  fn decode_data_uri_content(encoding: &str, content: &str, source: &BoxSource) -> Vec<u8> {
    if encoding == "base64"
      && let Some(cleaned) = rspack_base64::clean_base64(content)
    {
      return rspack_base64::decode_to_vec(cleaned.as_bytes())
        .unwrap_or_else(|_| source.buffer().to_vec());
    }

    match urlencoding::decode(content) {
      Ok(decoded_content) => decoded_content.as_bytes().to_vec(),
      Err(_) => content.as_bytes().to_vec(),
    }
  }

  fn hash_for_source(
    &self,
    source: &BoxSource,
    compiler_options: &CompilerOptions,
  ) -> RspackHashDigest {
    let mut hasher = RspackHash::from(&compiler_options.output);
    hasher.write(&source.buffer());
    hasher.digest(&compiler_options.output.hash_digest)
  }

  async fn get_data_url(
    &self,
    resource_data: &ResourceData,
    data_url: Option<&AssetGeneratorDataUrl>,
    source: &BoxSource,
    module: &dyn Module,
    compilation: &Compilation,
  ) -> Option<String> {
    let func_ctx = AssetGeneratorDataUrlFnCtx {
      filename: resource_data.resource.clone(),
      module,
      compilation,
    };

    if let Some(AssetGeneratorDataUrl::Func(data_url)) = data_url {
      return Some(
        data_url(source.buffer().to_vec(), func_ctx)
          .await
          .expect("call data_url function failed"),
      );
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
    if let Some(resource_path) = &resource_data.resource_path {
      return mime_guess::MimeGuess::from_path(resource_path)
        .first_raw()
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
          error!(
            "DataUrl can't be generated automatically, because there is no mimetype for \"{ext:?}\" in mimetype database. Either pass a mimetype via \"generator.mimetype\" or use type: \"asset/resource\" to create a resource file instead of a DataUrl",
            ext = resource_path.extension()
          )
        });
    }
    Err(error!("DataUrl can't be generated automatically. Either pass a mimetype via \"generator.mimetype\" or use type: \"asset/resource\" to create a resource file instead of a DataUrl"))
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
    if let Some(encoded_content) = &resource_data.encoded_content
      && let Some(resource_encoding) = &resource_data.encoding
      && resource_encoding == encoding
      && AssetParserAndGenerator::decode_data_uri_content(encoding, encoded_content, source)
        .eq(&source.buffer().to_vec())
    {
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

  async fn get_asset_module_filename(
    &self,
    module: &NormalModule,
    module_generator_options: Option<&GeneratorOptions>,
    compilation: &Compilation,
    contenthash: Option<&str>,
    source_file_name: &str,
    use_output_path: bool,
  ) -> Result<(String, String, AssetInfo)> {
    // Use [Rule.generator.filename] if it is set, otherwise use [output.assetModuleFilename].
    let asset_filename_template = module_generator_options
      .and_then(|x| x.asset_filename())
      .unwrap_or(&compilation.options.output.asset_module_filename);
    let path_data = PathData::default()
      .module_id_optional(
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.id())
          .map(|s| s.as_str()),
      )
      .content_hash_optional(contenthash)
      .hash_optional(contenthash)
      .filename(source_file_name);

    let (mut filename, mut asset_info) = compilation
      .get_asset_path_with_info(asset_filename_template, path_data)
      .await?;
    let original_filename = filename.clone();

    if use_output_path {
      let output_path = module_generator_options.and_then(|x| x.asset_output_path());

      if let Some(output_path) = output_path {
        let (output_path, another_asset_info) = compilation
          .get_asset_path_with_info(output_path, path_data)
          .await?;
        let output_path = PathBuf::from(output_path);
        let file_path = PathBuf::from(filename);
        filename = output_path.join(file_path).to_string_lossy().to_string();
        asset_info.merge_another_asset(another_asset_info);
      }
    }

    Ok((original_filename, filename, asset_info))
  }

  async fn get_public_path(
    &self,
    module: &NormalModule,
    compilation: &Compilation,
    contenthash: Option<&str>,
    source_file_name: &str,
    template: &Filename,
  ) -> Result<(String, AssetInfo)> {
    let (public_path, info) = compilation
      .get_asset_path_with_info(
        template,
        PathData::default()
          .module_id_optional(
            ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.id())
              .map(|s| s.as_str()),
          )
          .content_hash_optional(contenthash)
          .hash_optional(contenthash)
          .filename(source_file_name),
      )
      .await?;
    let public_path = PublicPath::ensure_ends_with_slash(public_path);
    Ok((public_path, info))
  }

  fn get_import_mode(
    &self,
    module_generator_options: Option<&GeneratorOptions>,
  ) -> Result<AssetGeneratorImportMode> {
    let import_mode = module_generator_options
      .and_then(|x| x.get_asset())
      .and_then(|x| x.import_mode)
      .or_else(|| {
        module_generator_options
          .and_then(|x| x.get_asset_resource())
          .and_then(|x| x.import_mode)
      })
      .unwrap_or_default();

    Ok(import_mode)
  }
}

// Webpack's default parser.dataUrlCondition.maxSize
const DEFAULT_MAX_SIZE: f64 = 8096.0;

#[cacheable_dyn]
#[async_trait::async_trait]
impl ParserAndGenerator for AssetParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    if let Some(config) = self.parsed_asset_config.as_ref() {
      if config.is_source() || config.is_inline() || !self.emit {
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

  fn size(&self, module: &dyn Module, source_type: Option<&SourceType>) -> f64 {
    let original_source_size = module.source().map_or(0, |source| source.size()) as f64;
    match source_type.unwrap_or(&SourceType::Asset) {
      SourceType::Asset => original_source_size,
      SourceType::JavaScript => {
        if module.source().is_none() {
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

  async fn parse<'a>(
    &mut self,
    parse_context: rspack_core::ParseContext<'a>,
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
      DataUrlOptions::Source => Some(CanonicalizedDataUrlOption::Source),
      DataUrlOptions::Inline(val) => Some(CanonicalizedDataUrlOption::Asset(*val)),
      DataUrlOptions::Auto(option) => {
        let limit_size = parse_context
          .module_parser_options
          .and_then(|x| {
            x.get_asset()
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

    Ok(
      rspack_core::ParseResult {
        // different from webpack
        // Rspack: when set asset as entry, output a js chunk with default export
        // webpack: Assets do not have dependencies
        dependencies: vec![Box::new(AssetExportsDependency::new())],
        blocks: vec![],
        source,
        presentational_dependencies: vec![],
        code_generation_dependencies: vec![],
        side_effects_bailout: None,
      }
      .with_empty_diagnostic(),
    )
  }

  async fn generate(
    &self,
    source: &BoxSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    let compilation = generate_context.compilation;
    let parsed_asset_config = self
      .parsed_asset_config
      .as_ref()
      .expect("should have parsed_asset_config in generate phase");
    let normal_module = module
      .as_normal_module()
      .expect("module should be a NormalModule in AssetParserAndGenerator");
    let module_generator_options = normal_module.get_generator_options();

    let import_mode = self.get_import_mode(module_generator_options)?;

    let result = match generate_context.requested_source_type {
      SourceType::JavaScript => {
        let exported_content = if parsed_asset_config.is_inline() {
          let resource_data: &ResourceData = normal_module.resource_resolved_data();
          let data_url = module_generator_options.and_then(|x| x.asset_data_url());
          let encoded_source: String;

          if let Some(custom_data_url) = self
            .get_data_url(resource_data, data_url, source, module, compilation)
            .await
          {
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

          serde_json::to_string(&encoded_source).to_rspack_result()?
        } else if parsed_asset_config.is_resource() {
          let contenthash = self.hash_for_source(source, &compilation.options);
          let contenthash = contenthash.rendered(compilation.options.output.hash_digest_length);

          let source_file_name = self.get_source_file_name(normal_module, compilation);
          let (original_filename, filename, mut asset_info) = self
            .get_asset_module_filename(
              normal_module,
              module_generator_options,
              compilation,
              Some(contenthash),
              &source_file_name,
              true,
            )
            .await?;

          let asset_path = if import_mode.is_preserve() {
            generate_context
              .data
              .insert(CodeGenerationPublicPathAutoReplace(true));
            serde_json::to_string(&format!(
              "{AUTO_PUBLIC_PATH_PLACEHOLDER}{original_filename}"
            ))
            .to_rspack_result()?
          } else if let Some(public_path) =
            module_generator_options.and_then(|x| x.asset_public_path())
          {
            let public_path = match public_path {
              PublicPath::Filename(template) => {
                let (public_path, another_asset_info) = self
                  .get_public_path(
                    normal_module,
                    compilation,
                    Some(contenthash),
                    &source_file_name,
                    template,
                  )
                  .await?;
                asset_info.merge_another_asset(another_asset_info);
                public_path
              }
              PublicPath::Auto => public_path.render(compilation, &filename).await,
            };
            serde_json::to_string(&format!("{public_path}{original_filename}"))
              .to_rspack_result()?
          } else {
            generate_context
              .runtime_requirements
              .insert(RuntimeGlobals::PUBLIC_PATH);
            format!(
              r#"{} + "{}""#,
              RuntimeGlobals::PUBLIC_PATH,
              original_filename
            )
          };

          asset_info.set_source_filename(source_file_name);

          generate_context
            .data
            .insert(CodeGenerationDataFilename::new(
              filename,
              match module_generator_options
                .and_then(|x| x.asset_public_path())
                .unwrap_or_else(|| &compilation.options.output.public_path)
              {
                PublicPath::Filename(p) => PublicPath::render_filename(compilation, p).await,
                PublicPath::Auto => AUTO_PUBLIC_PATH_PLACEHOLDER.to_string(),
              },
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

        if import_mode.is_preserve() && parsed_asset_config.is_resource() {
          let is_module = compilation.options.output.module;
          if let Some(ref mut scope) = generate_context.concatenation_scope {
            scope.register_namespace_export(NAMESPACE_OBJECT_EXPORT);
            if is_module {
              return Ok(
                RawStringSource::from(format!(
                  r#"import {NAMESPACE_OBJECT_EXPORT} from {exported_content};"#
                ))
                .boxed(),
              );
            } else {
              let supports_const = compilation.options.output.environment.supports_const();
              let declaration_kind = if supports_const { "const" } else { "var" };
              return Ok(
                RawStringSource::from(format!(
                  r#"{declaration_kind} {NAMESPACE_OBJECT_EXPORT} = require({exported_content});"#
                ))
                .boxed(),
              );
            }
          } else {
            generate_context
              .runtime_requirements
              .insert(RuntimeGlobals::MODULE);
            return Ok(
              RawStringSource::from(format!(r#"module.exports = require({exported_content});"#))
                .boxed(),
            );
          }
        };

        if let Some(ref mut scope) = generate_context.concatenation_scope {
          scope.register_namespace_export(NAMESPACE_OBJECT_EXPORT);
          let supports_const = compilation.options.output.environment.supports_const();
          let declaration_kind = if supports_const { "const" } else { "var" };
          Ok(
            RawStringSource::from(format!(
              r#"{declaration_kind} {NAMESPACE_OBJECT_EXPORT} = {exported_content};"#
            ))
            .boxed(),
          )
        } else {
          generate_context
            .runtime_requirements
            .insert(RuntimeGlobals::MODULE);

          Ok(RawStringSource::from(format!(r#"module.exports = {exported_content};"#)).boxed())
        }
      }
      SourceType::Asset => {
        if parsed_asset_config.is_source() || parsed_asset_config.is_inline() {
          Err(error!(
            "Inline or Source asset does not have source type `asset`"
          ))
        } else {
          Ok(source.clone().boxed())
        }
      }
      _ => panic!(
        "Unsupported source type: {:?}",
        generate_context.requested_source_type
      ),
    };

    result
  }

  fn get_concatenation_bailout_reason(
    &self,
    _module: &dyn rspack_core::Module,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    None
  }

  async fn get_runtime_hash(
    &self,
    module: &NormalModule,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    let parsed_asset_config = self
      .parsed_asset_config
      .as_ref()
      .expect("should have parsed_asset_config in generate phase");
    let module_generator_options = module.get_generator_options();
    if parsed_asset_config.is_inline()
      && let Some(AssetGeneratorDataUrl::Options(data_url_options)) =
        module_generator_options.and_then(|x| x.asset_data_url())
    {
      data_url_options.dyn_hash(&mut hasher);
    } else if parsed_asset_config.is_resource() {
      let source_file_name = self.get_source_file_name(module, compilation);
      let (filename, _, _) = self
        .get_asset_module_filename(
          module,
          module_generator_options,
          compilation,
          None,
          &source_file_name,
          false,
        )
        .await?;
      filename.dyn_hash(&mut hasher);
      match module_generator_options.and_then(|x| x.asset_public_path()) {
        Some(public_path) => match public_path {
          PublicPath::Filename(template) => {
            let (public_path, _) = self
              .get_public_path(module, compilation, None, &source_file_name, template)
              .await?;
            public_path.dyn_hash(&mut hasher);
          }
          PublicPath::Auto => "auto".dyn_hash(&mut hasher),
        },
        None => "no-path".dyn_hash(&mut hasher),
      };
    }
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

#[plugin_hook(CompilationRenderManifest for AssetPlugin)]
async fn render_manifest(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  manifest: &mut Vec<RenderManifestEntry>,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let module_graph = compilation.get_module_graph();

  let ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SourceType::Asset,
    &module_graph,
  );

  let assets = ordered_modules
    .par_iter()
    .map(|m| {
      let code_gen_result = compilation
        .code_generation_results
        .get(&m.identifier(), Some(chunk.runtime()));

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
        RenderManifestEntry {
          source: source.clone(),
          filename: asset_filename.to_owned(),
          has_filename: true,
          info: asset_info.to_owned(),
          auxiliary: true,
        }
      });

      Ok(result)
    })
    .collect::<Result<Vec<Option<RenderManifestEntry>>>>()?
    .into_par_iter()
    .flatten()
    .collect::<Vec<RenderManifestEntry>>();

  manifest.extend(assets);
  Ok(())
}

#[async_trait]
impl Plugin for AssetPlugin {
  fn name(&self) -> &'static str {
    "asset"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::Asset,
      Box::new(move |parser_options, generator_options| {
        let data_url_condition = parser_options
          .and_then(|x| x.get_asset())
          .and_then(|x| x.data_url_condition.clone());

        let emit: Option<bool> = generator_options
          .and_then(|x| x.get_asset())
          .and_then(|x| x.emit);

        Box::new(AssetParserAndGenerator::with_auto(
          data_url_condition.clone(),
          emit.unwrap_or(true),
        ))
      }),
    );

    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::AssetInline,
      Box::new(|_, _| Box::new(AssetParserAndGenerator::with_inline())),
    );

    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::AssetResource,
      Box::new(move |_, generator_options| {
        let emit = generator_options
          .and_then(|x| x.get_asset_resource())
          .and_then(|x| x.emit);

        Box::new(AssetParserAndGenerator::with_resource(emit.unwrap_or(true)))
      }),
    );

    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::AssetSource,
      Box::new(move |_, _| Box::new(AssetParserAndGenerator::with_source())),
    );

    Ok(())
  }
}

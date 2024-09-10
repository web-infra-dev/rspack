use std::borrow::Cow;
use std::iter;

use rspack_collections::{Identifiable, Identifier};
use rspack_error::{error, impl_empty_diagnosable_trait, Diagnostic, Result};
use rspack_macros::impl_source_map_config;
use rspack_util::{ext::DynHash, json_stringify, source_map::SourceMapKind};
use rustc_hash::{FxHashMap as HashMap, FxHashSet};
use serde::Serialize;

use crate::{
  extract_url_and_global, impl_module_meta_info, module_update_hash, property_access,
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  to_identifier, AsyncDependenciesBlockIdentifier, BuildContext, BuildInfo, BuildMeta,
  BuildMetaExportsType, BuildResult, ChunkInitFragments, ChunkUkey, CodeGenerationDataUrl,
  CodeGenerationResult, Compilation, ConcatenationScope, Context, DependenciesBlock, DependencyId,
  ExternalType, FactoryMeta, InitFragmentExt, InitFragmentKey, InitFragmentStage, LibIdentOptions,
  Module, ModuleType, NormalInitFragment, RuntimeGlobals, RuntimeSpec, SourceType,
  StaticExportsDependency, StaticExportsSpec, NAMESPACE_OBJECT_EXPORT,
};
use crate::{ChunkGraph, ModuleGraph};

static EXTERNAL_MODULE_JS_SOURCE_TYPES: &[SourceType] = &[SourceType::JavaScript];
static EXTERNAL_MODULE_CSS_SOURCE_TYPES: &[SourceType] = &[SourceType::CssImport];

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ExternalRequest {
  Single(ExternalRequestValue),
  Map(HashMap<String, ExternalRequestValue>),
}

#[derive(Debug, Clone)]
pub struct ExternalRequestValue {
  pub primary: String,
  rest: Option<Vec<String>>,
}

impl Serialize for ExternalRequestValue {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    if self.rest.is_none() {
      self.primary.serialize(serializer)
    } else {
      self.iter().collect::<Vec<_>>().serialize(serializer)
    }
  }
}

impl ExternalRequestValue {
  pub fn new(primary: String, rest: Option<Vec<String>>) -> Self {
    Self { primary, rest }
  }

  pub fn primary(&self) -> &str {
    &self.primary
  }

  pub fn rest(&self) -> Option<&[String]> {
    self.rest.as_deref()
  }

  pub fn iter(&self) -> impl Iterator<Item = &String> {
    if let Some(rest) = &self.rest {
      iter::once(&self.primary).chain(rest)
    } else {
      iter::once(&self.primary).chain(&[])
    }
  }
}

fn get_namespace_object_export(
  concatenation_scope: Option<&mut ConcatenationScope>,
  supports_const: bool,
) -> Cow<str> {
  if let Some(concatenation_scope) = concatenation_scope {
    concatenation_scope.register_namespace_export(NAMESPACE_OBJECT_EXPORT);
    format!(
      "{} {NAMESPACE_OBJECT_EXPORT}",
      if supports_const { "const" } else { "var" }
    )
    .into()
  } else {
    "module.exports".into()
  }
}

fn get_source_for_global_variable_external(
  variable_names: &ExternalRequestValue,
  external_type: &ExternalType,
) -> String {
  let object_lookup = property_access(variable_names.iter(), 0);
  format!("{external_type}{object_lookup}")
}

fn get_source_for_default_case(_optional: bool, request: &ExternalRequestValue) -> String {
  let variable_name = request.primary();
  let object_lookup = property_access(request.iter(), 1);
  format!("{variable_name}{object_lookup}")
}

fn get_source_for_commonjs(module_and_specifiers: &ExternalRequestValue) -> String {
  let module_name = module_and_specifiers.primary();
  format!(
    "require({}){}",
    json_stringify(module_name),
    property_access(module_and_specifiers.iter(), 1)
  )
}

fn get_source_for_import(
  module_and_specifiers: &ExternalRequestValue,
  compilation: &Compilation,
) -> String {
  format!(
    "{}({})",
    compilation.options.output.import_function_name,
    serde_json::to_string(module_and_specifiers.primary()).expect("invalid json to_string")
  )
}

#[impl_source_map_config]
#[derive(Debug)]
pub struct ExternalModule {
  dependencies: Vec<DependencyId>,
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  id: Identifier,
  pub request: ExternalRequest,
  pub external_type: ExternalType,
  /// Request intended by user (without loaders from config)
  pub user_request: String,
  factory_meta: Option<FactoryMeta>,
  build_info: Option<BuildInfo>,
  build_meta: Option<BuildMeta>,
  dependency_meta: DependencyMeta,
}

#[derive(Debug)]
pub enum ExternalTypeEnum {
  Import,
  Module,
}

pub type MetaExternalType = Option<ExternalTypeEnum>;

#[derive(Debug)]
pub struct DependencyMeta {
  pub external_type: MetaExternalType,
}

impl ExternalModule {
  pub fn new(
    request: ExternalRequest,
    external_type: ExternalType,
    user_request: String,
    dependency_meta: DependencyMeta,
  ) -> Self {
    Self {
      dependencies: Vec::new(),
      blocks: Vec::new(),
      id: Identifier::from(format!(
        "external {external_type} {}",
        serde_json::to_string(&request).expect("invalid json to_string")
      )),
      request,
      external_type,
      user_request,
      factory_meta: None,
      build_info: None,
      build_meta: None,
      source_map_kind: SourceMapKind::empty(),
      dependency_meta,
    }
  }

  pub fn get_external_type(&self) -> &ExternalType {
    &self.external_type
  }

  fn get_request_and_external_type(&self) -> (Option<&ExternalRequestValue>, &ExternalType) {
    match &self.request {
      ExternalRequest::Single(request) => (Some(request), &self.external_type),
      ExternalRequest::Map(map) => (map.get(&self.external_type), &self.external_type),
    }
  }

  fn get_source(
    &self,
    compilation: &Compilation,
    request: Option<&ExternalRequestValue>,
    external_type: &ExternalType,
    concatenation_scope: Option<&mut ConcatenationScope>,
  ) -> Result<(BoxSource, ChunkInitFragments, RuntimeGlobals)> {
    let mut chunk_init_fragments: ChunkInitFragments = Default::default();
    let mut runtime_requirements: RuntimeGlobals = Default::default();
    let supports_const = compilation.options.output.environment.supports_const();

    let source = match self.external_type.as_str() {
      "this" if let Some(request) = request => format!(
        "{} = (function() {{ return {}; }}());",
        get_namespace_object_export(concatenation_scope, supports_const),
        get_source_for_global_variable_external(request, external_type)
      ),
      "window" | "self" if let Some(request) = request => format!(
        "{} = {};",
        get_namespace_object_export(concatenation_scope, supports_const),
        get_source_for_global_variable_external(request, external_type)
      ),
      "global" if let Some(request) = request => format!(
        "{} = {};",
        get_namespace_object_export(concatenation_scope, supports_const),
        get_source_for_global_variable_external(request, &compilation.options.output.global_object)
      ),
      "commonjs" | "commonjs2" | "commonjs-module" | "commonjs-static"
        if let Some(request) = request =>
      {
        format!(
          "{} = {};",
          get_namespace_object_export(concatenation_scope, supports_const),
          get_source_for_commonjs(request)
        )
      }
      "node-commonjs" if let Some(request) = request => {
        if compilation.options.output.module {
          chunk_init_fragments.push(
            NormalInitFragment::new(
              "import { createRequire as __WEBPACK_EXTERNAL_createRequire } from \"module\";\n"
                .to_string(),
              InitFragmentStage::StageHarmonyImports,
              0,
              InitFragmentKey::ModuleExternal("node-commonjs".to_string()),
              None,
            )
            .boxed(),
          );
          format!(
            "{} = __WEBPACK_EXTERNAL_createRequire({}.url)({});",
            get_namespace_object_export(concatenation_scope, supports_const),
            compilation.options.output.import_meta_name,
            json_stringify(request.primary())
          )
        } else {
          format!(
            "{} = {};",
            get_namespace_object_export(concatenation_scope, supports_const),
            get_source_for_commonjs(request)
          )
        }
      }
      "amd" | "amd-require" | "umd" | "umd2" | "system" | "jsonp" => {
        let id = compilation
          .get_module_graph()
          .module_graph_module_by_identifier(&self.identifier())
          .map(|m| m.id(&compilation.chunk_graph))
          .unwrap_or_default();
        format!(
          "{} = __WEBPACK_EXTERNAL_MODULE_{}__;",
          get_namespace_object_export(concatenation_scope, supports_const),
          to_identifier(id)
        )
      }
      "module" | "import" | "module-import" if let Some(request) = request => {
        match self.get_module_import_type(external_type) {
          "import" => {
            format!(
              "{} = {};",
              get_namespace_object_export(concatenation_scope, supports_const),
              get_source_for_import(request, compilation)
            )
          }
          "module" => {
            if compilation.options.output.module {
              let id = to_identifier(&request.primary);
              chunk_init_fragments.push(
                NormalInitFragment::new(
                  format!(
                    "import * as __WEBPACK_EXTERNAL_MODULE_{}__ from {};\n",
                    id.clone(),
                    json_stringify(request.primary())
                  ),
                  InitFragmentStage::StageHarmonyImports,
                  0,
                  InitFragmentKey::ModuleExternal(request.primary().into()),
                  None,
                )
                .boxed(),
              );
              format!(
                r#"
{} = __WEBPACK_EXTERNAL_MODULE_{}__;
"#,
                get_namespace_object_export(concatenation_scope, supports_const),
                id.clone()
              )
            } else {
              format!(
                "{} = {};",
                get_namespace_object_export(concatenation_scope, supports_const),
                get_source_for_import(request, compilation)
              )
            }
          }
          r#type => panic!(
            "Unhandled external type: {} in \"module-import\" type",
            r#type
          ),
        }
      }
      "var" | "promise" | "const" | "let" | "assign" if let Some(request) = request => format!(
        "{} = {};",
        get_namespace_object_export(concatenation_scope, supports_const),
        get_source_for_default_case(false, request)
      ),
      "script" if let Some(request) = request => {
        let url_and_global = extract_url_and_global(request.primary())?;
        runtime_requirements.insert(RuntimeGlobals::LOAD_SCRIPT);
        format!(
          r#"
var __webpack_error__ = new Error();
{export} = new Promise(function(resolve, reject) {{
if(typeof {global} !== "undefined") return resolve();
{load_script}({url_str}, function(event) {{
  if(typeof {global} !== "undefined") return resolve();
  var errorType = event && (event.type === 'load' ? 'missing' : event.type);
  var realSrc = event && event.target && event.target.src;
  __webpack_error__.message = 'Loading script failed.\n(' + errorType + ': ' + realSrc + ')';
  __webpack_error__.name = 'ScriptExternalLoadError';
  __webpack_error__.type = errorType;
  __webpack_error__.request = realSrc;
  reject(__webpack_error__);
}}, {global_str});
}}).then(function() {{ return {global}; }});
"#,
          export = get_namespace_object_export(concatenation_scope, supports_const),
          global = url_and_global.global,
          global_str =
            serde_json::to_string(url_and_global.global).map_err(|e| error!(e.to_string()))?,
          url_str = serde_json::to_string(url_and_global.url).map_err(|e| error!(e.to_string()))?,
          load_script = RuntimeGlobals::LOAD_SCRIPT.name()
        )
      }
      _ => String::new(),
    };
    Ok((
      RawSource::from(source).boxed(),
      chunk_init_fragments,
      runtime_requirements,
    ))
  }

  fn get_module_import_type<'a>(&self, external_type: &'a ExternalType) -> &'a str {
    match external_type.as_str() {
      "module-import" => {
        if let Some(external_type) = self.dependency_meta.external_type.as_ref() {
          match external_type {
            ExternalTypeEnum::Import => "import",
            ExternalTypeEnum::Module => "module",
          }
        } else {
          "module"
        }
      }
      import_or_module => import_or_module,
    }
  }
}

impl Identifiable for ExternalModule {
  fn identifier(&self) -> Identifier {
    self.id
  }
}

impl DependenciesBlock for ExternalModule {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier) {
    self.blocks.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.blocks
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.push(dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[async_trait::async_trait]
impl Module for ExternalModule {
  impl_module_meta_info!();

  fn get_concatenation_bailout_reason(
    &self,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    match self.external_type.as_ref() {
      "amd" | "umd" | "amd-require" | "umd2" | "system" | "jsonp" => {
        // return `${this.externalType} externals can't be concatenated`;
        Some(format!("{} externals can't be concatenated", self.external_type).into())
      }
      _ => None,
    }
  }

  fn module_type(&self) -> &ModuleType {
    &ModuleType::JsAuto
  }

  fn get_diagnostics(&self) -> Vec<Diagnostic> {
    vec![]
  }

  fn source_types(&self) -> &[SourceType] {
    if self.external_type == "css-import" {
      EXTERNAL_MODULE_CSS_SOURCE_TYPES
    } else {
      EXTERNAL_MODULE_JS_SOURCE_TYPES
    }
  }

  fn chunk_condition(&self, chunk_key: &ChunkUkey, compilation: &Compilation) -> Option<bool> {
    if self.external_type == "css-import" {
      return Some(true);
    }
    Some(
      compilation
        .chunk_graph
        .get_number_of_entry_modules(chunk_key)
        > 0,
    )
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<str> {
    Cow::Owned(format!(
      "external {}",
      serde_json::to_string(&self.request).expect("invalid json to_string")
    ))
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: &Compilation) -> f64 {
    // copied from webpack `ExternalModule`
    // roughly for url
    42.0
  }

  async fn build(
    &mut self,
    _build_context: BuildContext<'_>,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    let (_, external_type) = self.get_request_and_external_type();

    let build_info = BuildInfo {
      top_level_declarations: Some(FxHashSet::default()),
      strict: true,
      ..Default::default()
    };

    let mut build_result = BuildResult {
      build_info,
      build_meta: Default::default(),
      dependencies: Vec::new(),
      blocks: Vec::new(),
      optimization_bailouts: vec![],
    };
    // TODO add exports_type for request
    match self.external_type.as_str() {
      "this" => build_result.build_info.strict = false,
      "system" => build_result.build_meta.exports_type = BuildMetaExportsType::Namespace,
      "script" | "promise" => build_result.build_meta.has_top_level_await = true,
      "module" | "import" | "module-import" => match self.get_module_import_type(external_type) {
        "module" => build_result.build_meta.exports_type = BuildMetaExportsType::Namespace,
        "import" => {
          build_result.build_meta.has_top_level_await = true;
          build_result.build_meta.exports_type = BuildMetaExportsType::Namespace;
        }
        _ => {}
      },
      _ => build_result.build_meta.exports_type = BuildMetaExportsType::Dynamic,
    }
    build_result
      .dependencies
      .push(Box::new(StaticExportsDependency::new(
        StaticExportsSpec::True,
        false,
      )));
    Ok(build_result)
  }

  #[tracing::instrument(name = "ExternalModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  fn code_generation(
    &self,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
    mut concatenation_scope: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut cgr = CodeGenerationResult::default();
    let (request, external_type) = self.get_request_and_external_type();
    match self.external_type.as_str() {
      "asset" if let Some(request) = request => {
        cgr.add(
          SourceType::JavaScript,
          RawSource::from(format!(
            "module.exports = {};",
            serde_json::to_string(request.primary()).map_err(|e| error!(e.to_string()))?
          ))
          .boxed(),
        );
        cgr
          .data
          .insert(CodeGenerationDataUrl::new(request.primary().to_string()));
      }
      "css-import" if let Some(request) = request => {
        cgr.add(
          SourceType::Css,
          RawSource::from(format!(
            "@import url({});",
            serde_json::to_string(request.primary()).map_err(|e| error!(e.to_string()))?
          ))
          .boxed(),
        );
      }
      _ => {
        let (source, chunk_init_fragments, runtime_requirements) = self.get_source(
          compilation,
          request,
          external_type,
          concatenation_scope.as_mut(),
        )?;
        cgr.add(SourceType::JavaScript, source);
        cgr.chunk_init_fragments = chunk_init_fragments;
        cgr.runtime_requirements.insert(runtime_requirements);
        cgr.set_hash(
          &compilation.options.output.hash_function,
          &compilation.options.output.hash_digest,
          &compilation.options.output.hash_salt,
        );
      }
    };
    if concatenation_scope.is_none() {
      cgr.runtime_requirements.insert(RuntimeGlobals::MODULE);
    }
    cgr.concatenation_scope = concatenation_scope;
    Ok(cgr)
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<str>> {
    Some(Cow::Borrowed(self.user_request.as_str()))
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<()> {
    self.id.dyn_hash(hasher);
    let is_optional = compilation.get_module_graph().is_optional(&self.id);
    is_optional.dyn_hash(hasher);
    module_update_hash(self, hasher, compilation, runtime);
    Ok(())
  }
}

impl_empty_diagnosable_trait!(ExternalModule);

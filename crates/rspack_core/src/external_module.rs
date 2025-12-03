use std::{borrow::Cow, hash::Hash, iter};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::{Identifiable, Identifier};
use rspack_error::{Result, ToStringResultToRspackResultExt, impl_empty_diagnosable_trait};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_macros::impl_source_map_config;
use rspack_util::{ext::DynHash, json_stringify, source_map::SourceMapKind};
use rustc_hash::{FxHashMap as HashMap, FxHashSet};
use serde::Serialize;

use crate::{
  AsyncDependenciesBlockIdentifier, BuildContext, BuildInfo, BuildMeta, BuildMetaExportsType,
  BuildResult, ChunkGraph, ChunkInitFragments, ChunkUkey, CodeGenerationDataUrl,
  CodeGenerationResult, Compilation, ConcatenationScope, Context, DependenciesBlock, DependencyId,
  ExternalType, FactoryMeta, ImportAttributes, InitFragmentExt, InitFragmentKey, InitFragmentStage,
  LibIdentOptions, Module, ModuleGraph, ModuleType, NAMESPACE_OBJECT_EXPORT, NormalInitFragment,
  PrefetchExportsInfoMode, RuntimeGlobals, RuntimeSpec, SourceType, StaticExportsDependency,
  StaticExportsSpec, UsedExports, extract_url_and_global, impl_module_meta_info,
  module_update_hash, property_access,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  to_identifier,
};

static EXTERNAL_MODULE_JS_SOURCE_TYPES: &[SourceType] = &[SourceType::JavaScript];
static EXTERNAL_MODULE_CSS_SOURCE_TYPES: &[SourceType] = &[SourceType::CssImport];
static EXTERNAL_MODULE_CSS_URL_SOURCE_TYPES: &[SourceType] = &[SourceType::CssUrl];
#[cacheable]
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ExternalRequest {
  Single(ExternalRequestValue),
  Map(HashMap<String, ExternalRequestValue>),
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ExternalRequestValue {
  pub primary: String,
  rest: Option<Vec<String>>,
}

impl ExternalRequestValue {
  pub fn has_rest(&self) -> bool {
    self.rest.as_ref().is_some_and(|r| !r.is_empty())
  }
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
) -> Cow<'_, str> {
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
  variable_names: Option<&ExternalRequestValue>,
  external_type: &ExternalType,
) -> String {
  let object_lookup = if let Some(variable_names) = variable_names {
    property_access(variable_names.iter(), 0)
  } else {
    "[undefined]".to_string()
  };
  format!("{external_type}{object_lookup}")
}

fn get_request_string(request: &ExternalRequestValue) -> String {
  let variable_name = request.primary();
  let object_lookup = property_access(request.iter(), 1);
  format!("{variable_name}{object_lookup}")
}

fn get_source_for_commonjs(module_and_specifiers: Option<&ExternalRequestValue>) -> String {
  let (module_name, properties) = if let Some(module_and_specifiers) = module_and_specifiers {
    (
      module_and_specifiers.primary(),
      property_access(module_and_specifiers.iter(), 1),
    )
  } else {
    ("undefined", String::new())
  };
  format!("require({}){}", json_stringify(module_name), properties)
}

fn get_source_for_import(
  module_and_specifiers: Option<&ExternalRequestValue>,
  compilation: &Compilation,
  attributes: &Option<ImportAttributes>,
) -> String {
  format!(
    "{}({}).then(function(module) {{ return module{}; }})",
    compilation.options.output.import_function_name,
    {
      let attributes_str = if let Some(attributes) = attributes {
        format!(
          ", {{ with: {} }}",
          serde_json::to_string(attributes).expect("invalid json to_string")
        )
      } else {
        String::new()
      };

      format!(
        "{}{}",
        if let Some(module_and_specifiers) = module_and_specifiers {
          serde_json::to_string(module_and_specifiers.primary()).expect("invalid json to_string")
        } else {
          "undefined".to_string()
        },
        attributes_str
      )
    },
    if let Some(module_and_specifiers) = module_and_specifiers {
      property_access(module_and_specifiers.iter(), 1)
    } else {
      String::new()
    }
  )
}

fn module_external_fragment_key(base: &str, attributes: &Option<ImportAttributes>) -> String {
  if let Some(attributes) = attributes {
    format!(
      "{}|{}",
      base,
      serde_json::to_string(attributes).expect("json stringify failed")
    )
  } else {
    base.to_string()
  }
}

/**
 * Resolve the detailed external type from the raw external type.
 * e.g. resolve "module" or "import" from "module-import" type
 */
fn resolve_external_type<'a>(
  external_type: &'a str,
  dependency_meta: &'a DependencyMeta,
) -> &'a str {
  match external_type {
    "commonjs-import" => {
      if let Some(ExternalTypeEnum::Import) = dependency_meta.external_type.as_ref() {
        "import"
      } else {
        "commonjs"
      }
    }
    "module-import" => {
      if let Some(external_type) = dependency_meta.external_type.as_ref() {
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

#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct ExternalModule {
  dependencies: Vec<DependencyId>,
  blocks: Vec<AsyncDependenciesBlockIdentifier>,
  pub id: Identifier,
  pub request: ExternalRequest,
  pub external_type: ExternalType,
  /// Request intended by user (without loaders from config)
  user_request: String,
  factory_meta: Option<FactoryMeta>,
  build_info: BuildInfo,
  build_meta: BuildMeta,
  dependency_meta: DependencyMeta,
  place_in_initial: bool,
}

#[cacheable]
#[derive(Debug)]
pub enum ExternalTypeEnum {
  Import,
  Module,
}

pub type MetaExternalType = Option<ExternalTypeEnum>;

#[cacheable]
#[derive(Debug)]
pub struct DependencyMeta {
  pub external_type: MetaExternalType,
  pub attributes: Option<ImportAttributes>,
  pub source_type: Option<SourceType>,
}

impl ExternalModule {
  pub fn new(
    request: ExternalRequest,
    external_type: ExternalType,
    user_request: String,
    dependency_meta: DependencyMeta,
    place_in_initial: bool,
  ) -> Self {
    Self {
      dependencies: Vec::new(),
      blocks: Vec::new(),
      id: Identifier::from({
        let resolved_type = resolve_external_type(external_type.as_str(), &dependency_meta);
        let request_str = serde_json::to_string(&request).expect("invalid json to_string");
        let attrs_str = dependency_meta
          .attributes
          .as_ref()
          .map_or(String::new(), |attrs| {
            format!(
              " {}",
              serde_json::to_string(attrs).expect("invalid json to_string")
            )
          });
        format!("external {resolved_type} {request_str}{attrs_str}")
      }),
      request,
      external_type,
      user_request,
      factory_meta: None,
      build_info: BuildInfo {
        top_level_declarations: Some(FxHashSet::default()),
        strict: true,
        ..Default::default()
      },
      build_meta: Default::default(),
      source_map_kind: SourceMapKind::empty(),
      dependency_meta,
      place_in_initial,
    }
  }

  pub fn user_request(&self) -> &str {
    &self.user_request
  }

  pub fn user_request_mut(&mut self) -> &mut String {
    &mut self.user_request
  }

  pub fn get_external_type(&self) -> &ExternalType {
    &self.external_type
  }

  pub fn get_request(&self) -> &ExternalRequestValue {
    match &self.request {
      ExternalRequest::Single(request) => request,
      ExternalRequest::Map(map) => &map[&self.external_type],
    }
  }

  fn get_request_and_external_type(&self) -> (Option<&ExternalRequestValue>, &ExternalType) {
    match &self.request {
      ExternalRequest::Single(request) => (Some(request), &self.external_type),
      ExternalRequest::Map(map) => (map.get(&self.external_type), &self.external_type),
    }
  }

  fn resolve_external_type(&self) -> &str {
    resolve_external_type(self.external_type.as_str(), &self.dependency_meta)
  }

  fn get_source(
    &self,
    compilation: &Compilation,
    request: Option<&ExternalRequestValue>,
    external_type: &ExternalType,
    runtime: Option<&RuntimeSpec>,
    concatenation_scope: Option<&mut ConcatenationScope>,
  ) -> Result<(BoxSource, ChunkInitFragments, RuntimeGlobals)> {
    let mut chunk_init_fragments: ChunkInitFragments = Default::default();
    let mut runtime_requirements: RuntimeGlobals = Default::default();
    let supports_const = compilation.options.output.environment.supports_const();
    let resolved_external_type = self.resolve_external_type();
    let module_graph = compilation.get_module_graph();
    let module_graph_cache = &compilation.module_graph_cache_artifact;

    let source = match resolved_external_type {
      "this" => format!(
        "{} = (function() {{ return {}; }}());",
        get_namespace_object_export(concatenation_scope, supports_const),
        get_source_for_global_variable_external(request, external_type),
      ),
      "window" | "self" => format!(
        "{} = {};",
        get_namespace_object_export(concatenation_scope, supports_const),
        get_source_for_global_variable_external(request, external_type)
      ),
      "global" => format!(
        "{} = {};",
        get_namespace_object_export(concatenation_scope, supports_const),
        get_source_for_global_variable_external(request, &compilation.options.output.global_object)
      ),
      "commonjs" | "commonjs2" | "commonjs-module" | "commonjs-static" => {
        format!(
          "{} = {};",
          get_namespace_object_export(concatenation_scope, supports_const),
          get_source_for_commonjs(request)
        )
      }
      "node-commonjs" => {
        let need_prefix = compilation
          .options
          .output
          .environment
          .supports_node_prefix_for_core_modules();

        if compilation.options.output.module {
          chunk_init_fragments.push(
            NormalInitFragment::new(
              format!(
                "import {{ createRequire as __rspack_createRequire }} from \"{}\";\n{} __rspack_createRequire_require = __rspack_createRequire({}.url);\n",
                if need_prefix { "node:module" } else { "module" },
                if compilation.options.output.environment.supports_const() {
                  "const"
                } else {
                  "var"
                },
                compilation.options.output.import_meta_name
              ),
              InitFragmentStage::StageESMImports,
              0,
              InitFragmentKey::ModuleExternal("node-commonjs".to_string()),
              None,
            )
            .boxed(),
          );
          let (request, specifiers) = if let Some(request) = request {
            (
              json_stringify(request.primary()),
              property_access(request.iter(), 1),
            )
          } else {
            ("undefined".to_string(), String::new())
          };
          format!(
            "{} = __rspack_createRequire_require({}){};",
            get_namespace_object_export(concatenation_scope, supports_const),
            request,
            specifiers
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
        let id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, self.identifier())
          .map(|s| s.as_str())
          .expect("should have module id");
        let external_variable = format!("__rspack_external_{}", to_identifier(id));
        let check_external_variable = if module_graph.is_optional(&self.id, module_graph_cache) {
          format!(
            "if(typeof {} === 'undefined') {{ {} }}\n",
            external_variable,
            compilation
              .runtime_template
              .throw_missing_module_error_block(&self.user_request)
          )
        } else {
          String::new()
        };
        format!(
          "{}{} = {};",
          check_external_variable,
          get_namespace_object_export(concatenation_scope, supports_const),
          external_variable
        )
      }
      "import" => format!(
        "{} = {};",
        get_namespace_object_export(concatenation_scope, supports_const),
        get_source_for_import(request, compilation, &self.dependency_meta.attributes)
      ),
      "var" | "promise" | "const" | "let" | "assign" => {
        let external_variable = if let Some(request) = request {
          get_request_string(request)
        } else {
          "undefined".to_string()
        };
        let check_external_variable = if module_graph.is_optional(&self.id, module_graph_cache)
          && let Some(request) = request
        {
          format!(
            "if(typeof {} === 'undefined') {{ {} }}\n",
            external_variable,
            compilation
              .runtime_template
              .throw_missing_module_error_block(&get_request_string(request))
          )
        } else {
          String::new()
        };
        format!(
          "{}{} = {};",
          check_external_variable,
          get_namespace_object_export(concatenation_scope, supports_const),
          external_variable
        )
      }
      "module" => {
        if compilation.options.output.module
          && let Some(request) = request
        {
          let id: Cow<'_, str> = if to_identifier(&request.primary) != request.primary
            || self.dependency_meta.attributes.is_some()
          {
            let mut hasher = RspackHash::from(&compilation.options.output);
            request.primary.hash(&mut hasher);
            if let Some(attributes) = &self.dependency_meta.attributes {
              serde_json::to_string(attributes)
                .expect("json stringify failed")
                .hash(&mut hasher);
            }
            let hash_suffix = hasher.digest(&compilation.options.output.hash_digest);
            Cow::Owned(format!(
              "{}_{}",
              to_identifier(&request.primary),
              hash_suffix.rendered(8)
            ))
          } else {
            to_identifier(&request.primary)
          };

          if let Some(concatenation_scope) = concatenation_scope {
            let exports_info = module_graph
              .get_prefetched_exports_info(&self.identifier(), PrefetchExportsInfoMode::Default);
            let used_exports = exports_info.get_used_exports(runtime);
            let meta = &self.dependency_meta.attributes;
            let attributes = meta.as_ref().map(|meta| {
              format!(
                " with {}",
                serde_json::to_string(meta).expect("json stringify failed"),
              )
            });

            #[derive(Clone, Copy)]
            struct ExternalImportOptimize(pub bool);

            let safe_to_optimize =
              if let Some(optimize) = concatenation_scope.data.get::<ExternalImportOptimize>() {
                optimize.0
              } else {
                let chunk = compilation
                  .chunk_graph
                  .get_module_chunks(concatenation_scope.concat_module_id);

                let safe_to_optimize = if chunk.is_empty() {
                  false
                } else {
                  let chunk = chunk
                    .iter()
                    .next()
                    .expect("concate module have only 1 chunk");

                  // chunk: [extern_1, extern_2],
                  // they are placed in 2 different concate modules
                  // They may have conflict symbols, but we can't know during codegen,
                  // and even if we know we can't determine which one to rename, as they
                  // render in parallel.
                  // This will be improved in incoming esm format.
                  //
                  // We must ensure that there is no other external modules in different
                  // concate modules of the same chunk
                  let module_graph = compilation.get_module_graph();
                  let mut safe_to_optimize = true;
                  'outer: for m in compilation
                    .chunk_graph
                    .get_chunk_modules(chunk, &module_graph)
                  {
                    if m.identifier() == concatenation_scope.concat_module_id {
                      // skip self
                      continue;
                    }

                    if let Some(m) = m.as_concatenated_module() {
                      for m in m.get_modules() {
                        if let Some(external_module) = module_graph
                          .module_by_identifier(&m.id)
                          .expect("should have module")
                          .as_external_module()
                          && external_module.resolve_external_type() == "module"
                        {
                          safe_to_optimize = false;
                          break 'outer;
                        }
                      }
                    }
                  }
                  safe_to_optimize
                };

                concatenation_scope
                  .data
                  .insert(ExternalImportOptimize(safe_to_optimize));
                safe_to_optimize
              };

            match used_exports {
              UsedExports::UsedNamespace(true) | UsedExports::Unknown => {
                chunk_init_fragments.push(
                  NormalInitFragment::new(
                    format!(
                      "import * as __rspack_external_{} from {}{};\n",
                      id.clone(),
                      json_stringify(request.primary()),
                      attributes.unwrap_or_default()
                    ),
                    InitFragmentStage::StageESMImports,
                    module_graph
                      .get_pre_order_index(&self.identifier())
                      .map_or(0, |num| num as i32),
                    InitFragmentKey::ModuleExternal(module_external_fragment_key(
                      request.primary(),
                      &self.dependency_meta.attributes,
                    )),
                    None,
                  )
                  .boxed(),
                );
                let external_module_id = format!("__rspack_external_{id}");
                let namespace_export_with_name = format!(
                  "{}{}{}",
                  NAMESPACE_OBJECT_EXPORT,
                  &external_module_id,
                  &property_access(request.iter(), 1)
                );
                concatenation_scope.register_namespace_export(&namespace_export_with_name);
              }
              UsedExports::UsedNamespace(false) => {
                let content = format!(
                  "import {}{};\n",
                  json_stringify(request.primary()),
                  attributes.unwrap_or_default()
                );
                let key = module_external_fragment_key(&content, &self.dependency_meta.attributes);
                chunk_init_fragments.push(
                  NormalInitFragment::new(
                    content,
                    InitFragmentStage::StageESMImports,
                    module_graph
                      .get_pre_order_index(&self.identifier())
                      .map_or(0, |num| num as i32),
                    InitFragmentKey::ModuleExternal(key),
                    None,
                  )
                  .boxed(),
                );
              }
              UsedExports::UsedNames(atoms) => {
                if !safe_to_optimize {
                  chunk_init_fragments.push(
                    NormalInitFragment::new(
                      format!(
                        "import * as __rspack_external_{} from {}{};\n",
                        id.clone(),
                        json_stringify(request.primary()),
                        attributes.clone().unwrap_or_default()
                      ),
                      InitFragmentStage::StageESMImports,
                      module_graph
                        .get_pre_order_index(&self.identifier())
                        .map_or(0, |num| num as i32),
                      InitFragmentKey::ModuleExternal(module_external_fragment_key(
                        request.primary(),
                        &self.dependency_meta.attributes,
                      )),
                      None,
                    )
                    .boxed(),
                  );
                  let external_module_id = format!("__rspack_external_{id}");
                  let namespace_export_with_name = format!(
                    "{}{}{}",
                    NAMESPACE_OBJECT_EXPORT,
                    &external_module_id,
                    &property_access(request.iter(), 1)
                  );
                  concatenation_scope.register_namespace_export(&namespace_export_with_name);
                } else {
                  concatenation_scope.register_import(
                    request.primary().to_string(),
                    attributes.clone(),
                    None,
                  );
                  for atom in &atoms {
                    concatenation_scope.register_import(
                      request.primary().to_string(),
                      attributes.clone(),
                      Some(atom.clone()),
                    );
                    concatenation_scope.register_raw_export(atom.clone(), atom.to_string());
                  }
                }
              }
            }

            String::new()
          } else {
            chunk_init_fragments.push(
              NormalInitFragment::new(
                format!(
                  "import * as __rspack_external_{} from {}{};\n",
                  id.clone(),
                  json_stringify(request.primary()),
                  {
                    let meta = &self.dependency_meta.attributes;
                    if let Some(meta) = meta {
                      format!(
                        " with {}",
                        serde_json::to_string(meta).expect("json stringify failed"),
                      )
                    } else {
                      String::new()
                    }
                  },
                ),
                InitFragmentStage::StageESMImports,
                0,
                InitFragmentKey::ModuleExternal(module_external_fragment_key(
                  request.primary(),
                  &self.dependency_meta.attributes,
                )),
                None,
              )
              .boxed(),
            );
            format!(
              r#"
{} = __rspack_external_{};
"#,
              get_namespace_object_export(concatenation_scope, supports_const),
              id.clone()
            )
          }
        } else {
          format!(
            "{} = {};",
            get_namespace_object_export(concatenation_scope, supports_const),
            get_source_for_import(request, compilation, &self.dependency_meta.attributes)
          )
        }
      }
      "script" if request.is_some() => {
        let request = request.expect("request should be some");
        let url_and_global = extract_url_and_global(request.primary())?;
        runtime_requirements.insert(RuntimeGlobals::LOAD_SCRIPT);
        format!(
          r#"
var __rspack_error = new Error();
{export} = new Promise(function(resolve, reject) {{
if(typeof {global} !== "undefined") return resolve();
{load_script}({url_str}, function(event) {{
  if(typeof {global} !== "undefined") return resolve();
  var errorType = event && (event.type === 'load' ? 'missing' : event.type);
  var realSrc = event && event.target && event.target.src;
  __rspack_error.message = 'Loading script failed.\n(' + errorType + ': ' + realSrc + ')';
  __rspack_error.name = 'ScriptExternalLoadError';
  __rspack_error.type = errorType;
  __rspack_error.request = realSrc;
  reject(__rspack_error);
}}, {global_str});
}}).then(function() {{ return {global}; }});
"#,
          export = get_namespace_object_export(concatenation_scope, supports_const),
          global = url_and_global.global,
          global_str = serde_json::to_string(url_and_global.global).to_rspack_result()?,
          url_str = serde_json::to_string(url_and_global.url).to_rspack_result()?,
          load_script = compilation
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::LOAD_SCRIPT)
        )
      }
      _ => {
        let external_variable = if let Some(request) = request {
          get_request_string(request)
        } else {
          "undefined".to_string()
        };
        let check_external_variable = if module_graph.is_optional(&self.id, module_graph_cache) {
          format!(
            "if(typeof {} === 'undefined') {{ {} }}\n",
            &external_variable,
            compilation
              .runtime_template
              .throw_missing_module_error_block(&external_variable)
          )
        } else {
          String::new()
        };
        format!(
          "{}{} = {};",
          check_external_variable,
          get_namespace_object_export(concatenation_scope, supports_const),
          external_variable,
        )
      }
    };
    Ok((
      RawStringSource::from(source).boxed(),
      chunk_init_fragments,
      runtime_requirements,
    ))
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

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|d| d != &dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependencies
  }
}

#[cacheable_dyn]
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
    &ModuleType::JsDynamic
  }

  fn source_types(&self, _module_graph: &ModuleGraph) -> &[SourceType] {
    if self.external_type == "asset"
      && self
        .dependency_meta
        .source_type
        .is_some_and(|t| t == SourceType::CssUrl)
    {
      EXTERNAL_MODULE_CSS_URL_SOURCE_TYPES
    } else if self.external_type == "css-import" {
      EXTERNAL_MODULE_CSS_SOURCE_TYPES
    } else {
      EXTERNAL_MODULE_JS_SOURCE_TYPES
    }
  }

  fn chunk_condition(&self, chunk_key: &ChunkUkey, compilation: &Compilation) -> Option<bool> {
    match self.external_type.as_str() {
      "css-import" | "module" | "import" | "module-import" if !self.place_in_initial => Some(true),
      _ => Some(
        compilation
          .chunk_graph
          .get_number_of_entry_modules(chunk_key)
          > 0,
      ),
    }
  }

  fn source(&self) -> Option<&BoxSource> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<'_, str> {
    Cow::Owned(format!(
      "external {}",
      serde_json::to_string(&self.request).expect("invalid json to_string")
    ))
  }

  fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
    // copied from webpack `ExternalModule`
    // roughly for url
    42.0
  }

  async fn build(
    &mut self,
    build_context: BuildContext,
    _: Option<&Compilation>,
  ) -> Result<BuildResult> {
    self.build_info.module = build_context.compiler_options.output.module;
    let resolved_external_type = self.resolve_external_type();
    let request = match &self.request {
      ExternalRequest::Single(request) => Some(request),
      ExternalRequest::Map(map) => map.get(&self.external_type),
    };
    let mut can_mangle = false;
    let mut exports_type = BuildMetaExportsType::Dynamic;
    match resolved_external_type {
      "this" => self.build_info.strict = false,
      "system" => {
        if !request.is_some_and(|r| r.has_rest()) {
          exports_type = BuildMetaExportsType::Namespace;
          can_mangle = true;
        }
      }
      "module" => {
        if self.build_info.module {
          if !request.is_some_and(|r| r.has_rest()) {
            exports_type = BuildMetaExportsType::Namespace;
            can_mangle = true;
          }
        } else {
          self.build_meta.has_top_level_await = true;
          if !request.is_some_and(|r| r.has_rest()) {
            exports_type = BuildMetaExportsType::Namespace;
            can_mangle = false;
          }
        }
      }
      "script" | "promise" => self.build_meta.has_top_level_await = true,
      "import" => {
        self.build_meta.has_top_level_await = true;
        if !request.is_some_and(|r| r.has_rest()) {
          exports_type = BuildMetaExportsType::Namespace;
          can_mangle = false;
        }
      }
      _ => {}
    }
    self.build_meta.exports_type = exports_type;
    Ok(BuildResult {
      dependencies: vec![Box::new(StaticExportsDependency::new(
        StaticExportsSpec::True,
        can_mangle,
      ))],
      blocks: Vec::new(),
      optimization_bailouts: vec![],
    })
  }

  // #[tracing::instrument("ExternalModule::code_generation", skip_all, fields(identifier = ?self.identifier()))]
  async fn code_generation(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
    mut concatenation_scope: Option<ConcatenationScope>,
  ) -> Result<CodeGenerationResult> {
    let mut cgr = CodeGenerationResult::default();
    let (request, external_type) = self.get_request_and_external_type();
    match self.external_type.as_str() {
      "asset" if request.is_some() => {
        let request = request.expect("request should be some");
        cgr.add(
          SourceType::JavaScript,
          RawStringSource::from(format!(
            "module.exports = {};",
            serde_json::to_string(request.primary()).to_rspack_result()?
          ))
          .boxed(),
        );
        cgr
          .data
          .insert(CodeGenerationDataUrl::new(request.primary().to_string()));
      }
      "css-import" if request.is_some() => {
        let request = request.expect("request should be some");
        cgr.add(
          SourceType::Css,
          RawStringSource::from(format!(
            "@import url({});",
            serde_json::to_string(request.primary()).to_rspack_result()?
          ))
          .boxed(),
        );
      }
      _ => {
        let (source, chunk_init_fragments, runtime_requirements) = self.get_source(
          compilation,
          request,
          external_type,
          runtime,
          concatenation_scope.as_mut(),
        )?;
        cgr.add(SourceType::JavaScript, source);
        cgr.chunk_init_fragments = chunk_init_fragments;
        cgr.runtime_requirements.insert(runtime_requirements);
      }
    };
    if concatenation_scope.is_none() {
      cgr.runtime_requirements.insert(RuntimeGlobals::MODULE);
    }
    cgr.concatenation_scope = concatenation_scope;
    Ok(cgr)
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<'_, str>> {
    Some(Cow::Borrowed(self.user_request.as_str()))
  }

  async fn get_runtime_hash(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    self.id.dyn_hash(&mut hasher);
    let is_optional = compilation
      .get_module_graph()
      .is_optional(&self.id, &compilation.module_graph_cache_artifact);
    is_optional.dyn_hash(&mut hasher);
    module_update_hash(self, &mut hasher, compilation, runtime);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

impl_empty_diagnosable_trait!(ExternalModule);

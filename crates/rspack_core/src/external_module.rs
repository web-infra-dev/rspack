use std::hash::Hash;
use std::{borrow::Cow, fmt};

use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_identifier::{Identifiable, Identifier};

use crate::{
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  to_identifier, AstOrSource, BuildContext, BuildMetaExportsType, BuildResult, ChunkInitFragments,
  CodeGenerationDataUrl, CodeGenerationResult, Compilation, Context, ExternalType,
  GenerationResult, InitFragment, InitFragmentStage, LibIdentOptions, Module, ModuleType,
  RuntimeGlobals, SourceType,
};

static EXTERNAL_MODULE_JS_SOURCE_TYPES: &[SourceType] = &[SourceType::JavaScript];
static EXTERNAL_MODULE_CSS_SOURCE_TYPES: &[SourceType] = &[SourceType::Css];

#[derive(Debug, Clone)]
pub struct ExternalRequest(pub Vec<String>);

impl fmt::Display for ExternalRequest {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self.0)
  }
}

impl ExternalRequest {
  pub fn as_str(&self) -> &str {
    // we're sure array have more than one element,because it is valid in js side
    self.0.get(0).expect("should have at least element")
  }
  pub fn as_array(&self) -> &Vec<String> {
    &self.0
  }
}
pub fn property_access(o: &Vec<String>, mut start: usize) -> String {
  let mut str = String::default();
  while start < o.len() {
    let property = &o[start];
    str.push_str(format!(r#"["{property}"]"#).as_str());
    start += 1;
  }
  str
}

fn get_source_for_global_variable_external(
  request: &ExternalRequest,
  external_type: &ExternalType,
) -> String {
  let object_lookup = property_access(request.as_array(), 0);
  format!("{external_type}{object_lookup}")
}

fn get_source_for_default_case(_optional: bool, request: &ExternalRequest) -> String {
  let request = request.as_array();
  let variable_name = request.get(0).expect("should have at least one element");
  let object_lookup = property_access(request, 1);
  format!("{variable_name}{object_lookup}")
}

#[derive(Debug)]
pub struct ExternalModule {
  id: Identifier,
  pub request: ExternalRequest,
  external_type: ExternalType,
  /// Request intended by user (without loaders from config)
  user_request: String,
}

impl ExternalModule {
  pub fn new(request: Vec<String>, external_type: ExternalType, user_request: String) -> Self {
    Self {
      id: Identifier::from(format!("external {external_type} {request:?}")),
      request: ExternalRequest(request),
      external_type,
      user_request,
    }
  }

  pub fn get_external_type(&self) -> &ExternalType {
    &self.external_type
  }

  fn get_source_for_commonjs(&self) -> String {
    let request = &self.request.as_array();
    let module_name = request.get(0).expect("should have at least one element");
    format!(
      "module.exports = require('{}'){}",
      module_name,
      property_access(request, 1)
    )
  }

  fn get_source_for_import(&self, compilation: &Compilation) -> String {
    format!(
      "module.exports = {}('{}')",
      compilation.options.output.import_function_name, self.request
    )
  }

  pub fn get_source(
    &self,
    compilation: &Compilation,
  ) -> (BoxSource, ChunkInitFragments, RuntimeGlobals) {
    let mut chunk_init_fragments: ChunkInitFragments = Default::default();
    let mut runtime_requirements: RuntimeGlobals = Default::default();
    let source = match self.external_type.as_str() {
      "this" => format!(
        "module.exports = (function() {{ return {}; }}())",
        get_source_for_global_variable_external(&self.request, &self.external_type)
      ),
      "window" | "self" => format!(
        "module.exports = {}",
        get_source_for_global_variable_external(&self.request, &self.external_type)
      ),
      "global" => format!(
        "module.exports ={} ",
        get_source_for_global_variable_external(
          &self.request,
          &compilation.options.output.global_object
        )
      ),
      "commonjs" | "commonjs2" | "commonjs-module" | "commonjs-static" => {
        self.get_source_for_commonjs()
      }
      "node-commonjs" => {
        if compilation.options.output.module {
          chunk_init_fragments
            .entry("external module node-commonjs".to_string())
            .or_insert(InitFragment::new(
              "import { createRequire as __WEBPACK_EXTERNAL_createRequire } from 'module';\n"
                .to_string(),
              InitFragmentStage::STAGE_HARMONY_IMPORTS,
              None,
            ));
          format!(
            "__WEBPACK_EXTERNAL_createRequire(import.meta.url)('{}')",
            self.request.as_str()
          )
        } else {
          self.get_source_for_commonjs()
        }
      }
      "amd" | "amd-require" | "umd" | "umd2" | "system" | "jsonp" => {
        let id = compilation
          .module_graph
          .module_graph_module_by_identifier(&self.identifier())
          .map(|m| m.id(&compilation.chunk_graph))
          .unwrap_or_default();
        format!(
          "module.exports = __WEBPACK_EXTERNAL_MODULE_{}__",
          to_identifier(id)
        )
      }
      "import" => self.get_source_for_import(compilation),
      "var" | "promise" | "const" | "let" | "assign" => {
        format!(
          "module.exports = {}",
          get_source_for_default_case(false, &self.request)
        )
      }
      "module" => {
        if compilation.options.output.module {
          let id = compilation
            .module_graph
            .module_graph_module_by_identifier(&self.identifier())
            .map(|m| m.id(&compilation.chunk_graph))
            .unwrap_or_default();
          let identifier = to_identifier(id);
          chunk_init_fragments
            .entry(format!("external module import {identifier}"))
            .or_insert(InitFragment::new(
              format!(
                "import * as __WEBPACK_EXTERNAL_MODULE_{identifier}__ from '{}';\n",
                self.request.as_str()
              ),
              InitFragmentStage::STAGE_HARMONY_IMPORTS,
              None,
            ));
          runtime_requirements.add(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
          format!(
            r#"var x = y => {{ var x = {{}}; {}(x, y); return x; }}
            var y = x => () => x
            module.exports = __WEBPACK_EXTERNAL_MODULE_{identifier}__"#,
            RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
          )
        } else {
          self.get_source_for_import(compilation)
        }
      }
      // TODO "script"
      _ => "".to_string(),
    };
    (
      RawSource::from(source).boxed(),
      chunk_init_fragments,
      runtime_requirements,
    )
  }
}

impl Identifiable for ExternalModule {
  fn identifier(&self) -> Identifier {
    self.id
  }
}

#[async_trait::async_trait]
impl Module for ExternalModule {
  fn module_type(&self) -> &ModuleType {
    &ModuleType::Js
  }

  fn source_types(&self) -> &[SourceType] {
    if self.external_type == "css-import" {
      EXTERNAL_MODULE_CSS_SOURCE_TYPES
    } else {
      EXTERNAL_MODULE_JS_SOURCE_TYPES
    }
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn readable_identifier(&self, _context: &Context) -> Cow<str> {
    Cow::Owned(format!("external {}", self.request))
  }

  fn size(&self, _source_type: &SourceType) -> f64 {
    // copied from webpack `ExternalModule`
    // roughly for url
    42.0
  }

  async fn build(
    &mut self,
    _build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    let mut build_result = BuildResult::default();
    // TODO add exports_type for request
    match self.external_type.as_str() {
      "this" => build_result.build_info.strict = false,
      "system" => build_result.build_meta.exports_type = BuildMetaExportsType::Namespace,
      "module" => build_result.build_meta.exports_type = BuildMetaExportsType::Namespace,
      "script" | "promise" => build_result.build_meta.is_async = true,
      "import" => {
        build_result.build_meta.is_async = true;
        build_result.build_meta.exports_type = BuildMetaExportsType::Namespace;
      }
      _ => build_result.build_meta.exports_type = BuildMetaExportsType::Dynamic,
    }
    Ok(build_result.with_empty_diagnostic())
  }

  fn code_generation(&self, compilation: &Compilation) -> Result<CodeGenerationResult> {
    let mut cgr = CodeGenerationResult::default();
    match self.external_type.as_str() {
      "asset" => {
        cgr.add(
          SourceType::JavaScript,
          GenerationResult::from(AstOrSource::from(
            RawSource::from(format!(
              "module.exports = {};",
              serde_json::to_string(&self.request.as_str())
                .map_err(|e| internal_error!(e.to_string()))?
            ))
            .boxed(),
          )),
        );
        cgr.data.insert(CodeGenerationDataUrl::new(
          self.request.as_str().to_string(),
        ));
      }
      "css-import" => {
        cgr.add(
          SourceType::Css,
          GenerationResult::from(AstOrSource::from(
            RawSource::from(format!(
              "@import url({});",
              serde_json::to_string(&self.request.as_str())
                .map_err(|e| internal_error!(e.to_string()))?
            ))
            .boxed(),
          )),
        );
      }
      _ => {
        let (source, chunk_init_fragments, runtime_requirements) = self.get_source(compilation);
        cgr.add(
          SourceType::JavaScript,
          GenerationResult::from(AstOrSource::from(source)),
        );
        cgr.chunk_init_fragments = chunk_init_fragments;
        cgr.runtime_requirements.add(runtime_requirements);
        cgr.set_hash();
      }
    };
    Ok(cgr)
  }

  fn lib_ident(&self, _options: LibIdentOptions) -> Option<Cow<str>> {
    Some(Cow::Borrowed(self.user_request.as_str()))
  }
}

impl Hash for ExternalModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__ExternalModule".hash(state);
    self.identifier().hash(state);
  }
}

impl PartialEq for ExternalModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for ExternalModule {}

use std::borrow::Cow;
use std::hash::Hash;

use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_identifier::{Identifiable, Identifier};

use crate::{
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  to_identifier, AstOrSource, BuildContext, BuildResult, CodeGenerationResult, Compilation,
  Context, ExternalType, GenerationResult, LibIdentOptions, Module, ModuleType, SourceType,
};

static EXTERNAL_MODULE_SOURCE_TYPES: &[SourceType] = &[SourceType::JavaScript];

#[derive(Debug)]
pub struct ExternalModule {
  pub request: String,
  external_type: ExternalType,
  /// Request intended by user (without loaders from config)
  user_request: String,
}

impl ExternalModule {
  pub fn new(request: String, external_type: ExternalType, user_request: String) -> Self {
    Self {
      request,
      external_type,
      user_request,
    }
  }

  pub fn get_source(&self, compilation: &Compilation) -> BoxSource {
    let source = match self.external_type.as_str() {
      "this" => format!(
        "module.exports = (function() {{ return this['{}']; }}())",
        self.request
      ),
      "window" | "self" => format!(
        "module.exports = {}['{}']",
        self.external_type, self.request
      ),
      "global" => format!(
        "module.exports = {}['{}']",
        compilation.options.output.global_object, self.request
      ),
      "commonjs" | "commonjs2" | "commonjs-module" | "commonjs-static" => {
        format!("module.exports = require('{}')", self.request)
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
      "import" => {
        format!(
          "module.exports = {}('{}')",
          compilation.options.output.import_function_name, self.request
        )
      }
      "var" | "promise" | "const" | "let" | "assign" => {
        format!("module.exports = {}", self.request)
      }
      // TODO "script" "module"
      _ => "".to_string(),
    };
    RawSource::from(source).boxed()
  }
}

impl Identifiable for ExternalModule {
  fn identifier(&self) -> Identifier {
    let id = format!("external {} {}", self.external_type, self.request);
    Identifier::from(id.as_str())
  }
}

#[async_trait::async_trait]
impl Module for ExternalModule {
  fn module_type(&self) -> &ModuleType {
    &ModuleType::Js
  }

  fn source_types(&self) -> &[SourceType] {
    EXTERNAL_MODULE_SOURCE_TYPES
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
    Ok(BuildResult::default().with_empty_diagnostic())
  }

  fn code_generation(&self, compilation: &Compilation) -> Result<CodeGenerationResult> {
    let mut cgr = CodeGenerationResult::default();

    cgr.add(
      SourceType::JavaScript,
      GenerationResult::from(AstOrSource::from(self.get_source(compilation))),
    );

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

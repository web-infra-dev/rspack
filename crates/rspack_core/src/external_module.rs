use std::borrow::Cow;
use std::hash::Hash;

use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_identifier::{Identifiable, Identifier};

use crate::{
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  AstOrSource, BuildContext, BuildResult, CodeGenerationResult, Compilation, Context, ExternalType,
  LibIdentOptions, Module, ModuleType, SourceType, Target, TargetPlatform,
};

static EXTERNAL_MODULE_SOURCE_TYPES: &[SourceType] = &[SourceType::JavaScript];

#[derive(Debug)]
pub struct ExternalModule {
  pub request: String,
  external_type: ExternalType,
  target: Target,

  cached_source: Option<BoxSource>,
  /// Request intended by user (without loaders from config)
  user_request: String,
}

impl ExternalModule {
  pub fn new(
    request: String,
    external_type: ExternalType,
    target: Target,
    user_request: String,
  ) -> Self {
    Self {
      request,
      external_type,
      target,

      cached_source: None,
      user_request,
    }
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
    if self.cached_source.is_none() {
      let source = RawSource::from(match self.external_type {
        ExternalType::NodeCommonjs => {
          format!(r#"module.exports = require("{}")"#, self.request)
        }
        ExternalType::Window => {
          format!(r#"module.exports = window["{}"]"#, self.request)
        }
        ExternalType::Auto => match self.target.platform {
          TargetPlatform::Web | TargetPlatform::WebWorker | TargetPlatform::None => {
            format!("module.exports = {}", self.request)
          }
          TargetPlatform::Node(_) => {
            format!(r#"module.exports = require("{}")"#, self.request)
          }
        },
      })
      .boxed();

      self.cached_source = Some(source);
    }

    Ok(BuildResult::default().with_empty_diagnostic())
  }

  fn code_generation(&self, _compilation: &Compilation) -> Result<CodeGenerationResult> {
    let mut cgr = CodeGenerationResult::default();
    let source: AstOrSource = self
      .cached_source
      .as_ref()
      .ok_or_else(|| internal_error!("Source should exist"))?
      .clone()
      .into();
    cgr.add(SourceType::JavaScript, source);

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

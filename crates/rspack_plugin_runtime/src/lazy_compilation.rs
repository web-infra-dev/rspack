use std::borrow::Cow;
use std::hash::Hash;

use async_trait::async_trait;
use rspack_core::{
  rspack_sources::{RawSource, Source, SourceExt},
  runtime_globals, ApplyContext, AstOrSource, Compilation, DependencyType, Module, ModuleArgs,
  ModuleType, Plugin, PluginContext, PluginModuleHookOutput, SourceType,
};
use rspack_core::{BuildContext, BuildResult, CodeGenerationResult, Context, ModuleIdentifier};
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_identifier::Identifiable;

#[derive(Debug)]
pub struct LazyCompilationProxyModule {
  pub module_identifier: ModuleIdentifier,
}

#[async_trait::async_trait]
impl Module for LazyCompilationProxyModule {
  fn module_type(&self) -> &ModuleType {
    &ModuleType::Js
  }

  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn original_source(&self) -> Option<&dyn Source> {
    None
  }

  fn readable_identifier(&self, context: &Context) -> Cow<str> {
    Cow::Owned(context.shorten(&self.identifier()))
  }

  fn size(&self, _source_type: &SourceType) -> f64 {
    200.0
  }

  async fn build(
    &mut self,
    _build_context: BuildContext<'_>,
  ) -> Result<TWithDiagnosticArray<BuildResult>> {
    Ok(
      BuildResult {
        hash: Default::default(),
        cacheable: true,
        dependencies: vec![],
        file_dependencies: Default::default(),
        context_dependencies: Default::default(),
        missing_dependencies: Default::default(),
        build_dependencies: Default::default(),
      }
      .with_empty_diagnostic(),
    )
  }

  fn code_generation(&self, _compilation: &Compilation) -> Result<CodeGenerationResult> {
    let mut cgr = CodeGenerationResult::default();
    cgr.add(
      SourceType::JavaScript,
      AstOrSource::Source(
        RawSource::from(
          include_str!("runtime/lazy_compilation.js")
            // TODO
            .replace("$CHUNK_ID$", self.module_identifier.to_string().as_str())
            .replace("$MODULE_ID$", self.module_identifier.to_string().as_str()),
        )
        .boxed(),
      ),
    );
    cgr
      .runtime_requirements
      .insert(runtime_globals::LOAD_SCRIPT);
    Ok(cgr)
  }
}

impl Identifiable for LazyCompilationProxyModule {
  fn identifier(&self) -> ModuleIdentifier {
    self.module_identifier
  }
}

impl Hash for LazyCompilationProxyModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    "__rspack_internal__LazyCompilationProxyModule".hash(state);
    self.identifier().hash(state);
  }
}

impl PartialEq for LazyCompilationProxyModule {
  fn eq(&self, other: &Self) -> bool {
    self.identifier() == other.identifier()
  }
}

impl Eq for LazyCompilationProxyModule {}

#[derive(Debug)]
pub struct LazyCompilationPlugin {}

#[async_trait]
impl Plugin for LazyCompilationPlugin {
  fn name(&self) -> &'static str {
    "LazyCompilationPlugin"
  }

  fn apply(&mut self, _ctx: PluginContext<&mut ApplyContext>) -> Result<()> {
    Ok(())
  }

  async fn module(&self, _ctx: PluginContext, args: &ModuleArgs) -> PluginModuleHookOutput {
    if args.indentfiler.contains("rspack-dev-client")
      || args.lazy_visit_modules.contains(args.indentfiler.as_str())
    {
      return Ok(None);
    }
    if matches!(
      args.dependency_type,
      DependencyType::DynamicImport | DependencyType::Entry
    ) {
      return Ok(Some(Box::new(LazyCompilationProxyModule {
        module_identifier: args.indentfiler,
      })));
    }

    Ok(None)
  }
}

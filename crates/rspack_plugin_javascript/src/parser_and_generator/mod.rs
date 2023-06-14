use rspack_core::rspack_sources::{
  MapOptions, RawSource, Source, SourceExt, SourceMap, SourceMapSource, SourceMapSourceOptions,
};
use rspack_core::{
  AstOrSource, GenerateContext, GenerationResult, Module, ModuleAst, ParseContext, ParseResult,
  ParserAndGenerator, SourceType,
};
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_core::common::util::take::Take;
use swc_core::ecma::ast;
mod string_replace;
pub use string_replace::JavaScriptStringReplaceParserAndGenerator;

use crate::utils::syntax_by_module_type;
use crate::visitors::{run_after_pass, run_before_pass, scan_dependencies};

#[derive(Debug)]
pub struct JavaScriptParserAndGenerator;

impl JavaScriptParserAndGenerator {
  pub(crate) fn new() -> Self {
    Self {}
  }
}

static SOURCE_TYPES: &[SourceType; 1] = &[SourceType::JavaScript];

impl ParserAndGenerator for JavaScriptParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    SOURCE_TYPES
  }

  fn size(&self, module: &dyn Module, _source_type: &SourceType) -> f64 {
    module.original_source().map_or(0, |source| source.size()) as f64
  }

  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      module_type,
      resource_data,
      compiler_options,
      build_info,
      build_meta,
      ..
    } = parse_context;

    let syntax = syntax_by_module_type(
      &resource_data.resource_path,
      module_type,
      compiler_options.builtins.decorator.is_some(),
    );
    let source = source.source();
    let (mut ast, diagnostics) = match crate::ast::parse(
      source.to_string(),
      syntax,
      &resource_data.resource_path.to_string_lossy(),
      module_type,
    ) {
      Ok(ast) => (ast, Vec::new()),
      Err(diagnostics) => (
        rspack_core::ast::javascript::Ast::new(
          ast::Program::Module(ast::Module::dummy()),
          Default::default(),
          None,
        ),
        diagnostics.into(),
      ),
    };

    run_before_pass(
      resource_data,
      &mut ast,
      compiler_options,
      syntax,
      build_info,
      module_type,
      &source,
    )?;

    let (dependencies, presentational_dependencies, code_replace_source_dependencies) =
      ast.visit(|program, context| {
        scan_dependencies(
          program,
          context.unresolved_mark,
          resource_data,
          compiler_options,
          module_type,
          build_info,
          build_meta,
        )
      });

    Ok(
      ParseResult {
        ast_or_source: ModuleAst::JavaScript(ast).into(),
        dependencies,
        presentational_dependencies,
        code_replace_source_dependencies,
      }
      .with_diagnostic(diagnostics),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    ast_or_source: &AstOrSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<GenerationResult> {
    if matches!(
      generate_context.requested_source_type,
      SourceType::JavaScript
    ) {
      // TODO: this should only return AST for javascript only, It's a fast pass, defer to another pr to solve this.
      // Ok(ast_or_source.to_owned().into())
      let mut ast = ast_or_source
        .to_owned()
        .try_into_ast()?
        .try_into_javascript()?;
      run_after_pass(&mut ast, module, generate_context)?;
      let keep_comments = generate_context
        .compilation
        .options
        .builtins
        .code_generation
        .as_ref()
        .map(|cg| cg.keep_comments);
      let output = crate::ast::stringify(
        &ast,
        &generate_context.compilation.options.devtool,
        keep_comments,
      )?;
      if let Some(map) = output.map {
        Ok(GenerationResult {
          ast_or_source: SourceMapSource::new(SourceMapSourceOptions {
            value: output.code,
            source_map: SourceMap::from_json(&map).map_err(|e| internal_error!(e.to_string()))?,
            name: module.try_as_normal_module()?.user_request().to_string(),
            original_source: {
              Some(
                // Safety: you can sure that `build` is called before code generation, so that the `original_source` is exist
                module
                  .original_source()
                  .expect("Failed to get original source, please file an issue.")
                  .source()
                  .to_string(),
              )
            },
            inner_source_map: {
              // Safety: you can sure that `build` is called before code generation, so that the `original_source` is exist
              module
                .original_source()
                .expect("Failed to get original source, please file an issue.")
                .map(&MapOptions::default())
            },
            remove_original_source: false,
          })
          .boxed()
          .into(),
        })
      } else {
        Ok(GenerationResult {
          ast_or_source: RawSource::from(output.code).boxed().into(),
        })
      }
    } else {
      Err(internal_error!(
        "Unsupported source type {:?} for plugin JavaScript",
        generate_context.requested_source_type,
      ))
    }
  }
}

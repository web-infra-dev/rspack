use rspack_core::rspack_sources::{
  RawSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource, WithoutOriginalOptions,
};
use rspack_core::{
  AstOrSource, CodeGeneratableContext, GenerateContext, GenerationResult, Module, ModuleAst,
  ParseContext, ParseResult, ParserAndGenerator, SourceType,
};
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};

use crate::runtime::render_init_fragments;
use crate::utils::syntax_by_module_type;
use crate::visitors::{run_before_pass, scan_dependencies, swc_visitor::resolver};
#[derive(Debug)]
pub struct JavaScriptParserAndGenerator;

#[allow(unused)]
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
      module_identifier,
      ..
    } = parse_context;

    let syntax = syntax_by_module_type(
      &resource_data.resource_path,
      module_type,
      compiler_options.builtins.decorator.is_some(),
    );
    let source = source.source();
    let mut ast = match crate::ast::parse(
      source.to_string(),
      syntax,
      &resource_data.resource_path.to_string_lossy(),
      module_type,
    ) {
      Ok(ast) => ast,
      Err(e) => {
        return Ok(
          ParseResult {
            ast_or_source: RawSource::from(source.to_string()).boxed().into(),
            dependencies: vec![],
            presentational_dependencies: vec![],
          }
          .with_diagnostic(e.into()),
        );
      }
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

    let output: crate::TransformOutput =
      crate::ast::stringify(&ast, &compiler_options.devtool, Some(true))?;

    let mut scan_ast = match crate::ast::parse(
      output.code.clone(), // TODO avoid code clone
      syntax,
      &resource_data.resource_path.to_string_lossy(),
      module_type,
    ) {
      Ok(ast) => ast,
      Err(e) => {
        return Ok(
          ParseResult {
            ast_or_source: RawSource::from(output.code.clone()).boxed().into(),
            dependencies: vec![],
            presentational_dependencies: vec![],
          }
          .with_diagnostic(e.into()),
        );
      }
    };

    scan_ast.transform(|program, context| {
      program.visit_mut_with(&mut resolver(
        context.unresolved_mark,
        context.top_level_mark,
        syntax.typescript(),
      ));
    });

    let (dependencies, presentational_dependencies) = scan_ast.visit(|program, context| {
      scan_dependencies(
        program,
        context.unresolved_mark,
        resource_data,
        compiler_options,
        module_type,
        build_info,
        build_meta,
        module_identifier,
      )
    });

    let source = if let Some(map) = output.map {
      SourceMapSource::new(WithoutOriginalOptions {
        value: output.code,
        name: resource_data.resource_path.to_string_lossy().to_string(),
        source_map: SourceMap::from_json(&map).map_err(|e| internal_error!(e.to_string()))?,
      })
      .boxed()
    } else {
      RawSource::from(output.code).boxed()
    };

    Ok(
      ParseResult {
        ast_or_source: AstOrSource::new(Some(ModuleAst::JavaScript(scan_ast)), Some(source)),
        dependencies,
        presentational_dependencies,
      }
      .with_empty_diagnostic(),
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
      let mut source = ReplaceSource::new(ast_or_source.try_to_source()?);
      let compilation = generate_context.compilation;
      let mut init_fragments = vec![];
      let mut context = CodeGeneratableContext {
        compilation,
        module,
        runtime_requirements: generate_context.runtime_requirements,
        init_fragments: &mut init_fragments,
      };

      let mgm = compilation
        .module_graph
        .module_graph_module_by_identifier(&module.identifier())
        .expect("should have module graph module");

      mgm.dependencies.iter().for_each(|id| {
        if let Some(dependency) = compilation
          .module_graph
          .dependency_by_id(id)
          .expect("should have dependency")
          .as_code_generatable_dependency()
        {
          dependency.apply(&mut source, &mut context)
        }
      });

      if let Some(dependencies) = module.get_presentational_dependencies() {
        dependencies
          .iter()
          .for_each(|dependency| dependency.apply(&mut source, &mut context));
      };

      Ok(GenerationResult {
        ast_or_source: render_init_fragments(source.boxed(), &mut init_fragments).into(),
      })
    } else {
      Err(internal_error!(
        "Unsupported source type {:?} for plugin JavaScript",
        generate_context.requested_source_type,
      ))
    }
  }
}

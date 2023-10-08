use rspack_core::rspack_sources::{
  BoxSource, MapOptions, OriginalSource, RawSource, ReplaceSource, Source, SourceExt, SourceMap,
  SourceMapSource, SourceMapSourceOptions,
};
use rspack_core::tree_shaking::analyzer::OptimizeAnalyzer;
use rspack_core::tree_shaking::js_module::JsModule;
use rspack_core::tree_shaking::visitor::OptimizeAnalyzeResult;
use rspack_core::{
  render_init_fragments, GenerateContext, Module, ParseContext, ParseResult, ParserAndGenerator,
  SourceType, TemplateContext,
};
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_core::common::SyntaxContext;

use crate::utils::syntax_by_module_type;
use crate::visitors::{run_before_pass, scan_dependencies, swc_visitor::resolver};
use crate::{SideEffectsFlagPluginVisitor, SyntaxContextInfo};
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
      compiler_options.should_transform_by_default(),
    );
    let use_source_map = compiler_options.devtool.enabled();
    let use_simple_source_map = compiler_options.devtool.source_map();
    let original_map = source.map(&MapOptions::new(!compiler_options.devtool.cheap()));
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
            source: create_source(
              source.to_string(),
              resource_data.resource_path.to_string_lossy().to_string(),
              use_simple_source_map,
            ),
            dependencies: vec![],
            presentational_dependencies: vec![],
            analyze_result: Default::default(),
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

    ast = match crate::ast::parse(
      output.code.clone(),
      syntax,
      &resource_data.resource_path.to_string_lossy(),
      module_type,
    ) {
      Ok(ast) => ast,
      Err(e) => {
        return Ok(
          ParseResult {
            source: create_source(
              source.to_string(),
              resource_data.resource_path.to_string_lossy().to_string(),
              use_simple_source_map,
            ),
            dependencies: vec![],
            presentational_dependencies: vec![],
            analyze_result: Default::default(),
          }
          .with_diagnostic(e.into()),
        );
      }
    };

    ast.transform(|program, context| {
      program.visit_mut_with(&mut resolver(
        context.unresolved_mark,
        context.top_level_mark,
        syntax.typescript(),
      ));
    });

    let (dependencies, presentational_dependencies) = ast.visit(|program, context| {
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

    let analyze_result = if compiler_options.builtins.tree_shaking.enable() {
      JsModule::new(&ast, &dependencies, module_identifier, compiler_options).analyze()
    } else {
      OptimizeAnalyzeResult::default()
    };

    if compiler_options.is_new_tree_shaking()
      && compiler_options.optimization.side_effects.is_true()
    {
      ast.transform(|program, context| {
        let unresolved_ctxt = SyntaxContext::empty().apply_mark(context.unresolved_mark);
        let mut visitor =
          SideEffectsFlagPluginVisitor::new(SyntaxContextInfo::new(unresolved_ctxt));
        program.visit_with(&mut visitor);
        build_meta.side_effect_free = Some(visitor.side_effects_span.is_none());
      });
    }

    let source = if let Some(map) = output.map {
      SourceMapSource::new(SourceMapSourceOptions {
        value: output.code,
        name: resource_data.resource_path.to_string_lossy().to_string(),
        source_map: SourceMap::from_json(&map).map_err(|e| internal_error!(e.to_string()))?,
        inner_source_map: use_source_map.then_some(original_map).flatten(),
        remove_original_source: true,
        ..Default::default()
      })
      .boxed()
    } else if use_simple_source_map {
      OriginalSource::new(output.code, resource_data.resource_path.to_string_lossy()).boxed()
    } else {
      RawSource::from(output.code).boxed()
    };

    fn create_source(content: String, resource_path: String, devtool: bool) -> BoxSource {
      if devtool {
        return OriginalSource::new(content, resource_path).boxed();
      }
      RawSource::from(content).boxed()
    }

    Ok(
      ParseResult {
        source,
        dependencies,
        presentational_dependencies,
        analyze_result,
      }
      .with_empty_diagnostic(),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    source: &BoxSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    if matches!(
      generate_context.requested_source_type,
      SourceType::JavaScript
    ) {
      let mut source = ReplaceSource::new(source.clone());
      let compilation = generate_context.compilation;
      let mut init_fragments = vec![];
      let mut context = TemplateContext {
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
          .as_dependency_template()
        {
          dependency.apply(&mut source, &mut context)
        }
      });

      if let Some(dependencies) = module.get_presentational_dependencies() {
        dependencies
          .iter()
          .for_each(|dependency| dependency.apply(&mut source, &mut context));
      };

      Ok(render_init_fragments(
        source.boxed(),
        init_fragments,
        generate_context,
      ))
    } else {
      Err(internal_error!(
        "Unsupported source type {:?} for plugin JavaScript",
        generate_context.requested_source_type,
      ))
    }
  }
}

use rspack_core::diagnostics::map_box_diagnostics_to_module_parse_diagnostics;
use rspack_core::needs_refactor::WorkerSyntaxList;
use rspack_core::rspack_sources::{BoxSource, ReplaceSource, Source, SourceExt};
use rspack_core::tree_shaking::analyzer::OptimizeAnalyzer;
use rspack_core::tree_shaking::js_module::JsModule;
use rspack_core::tree_shaking::visitor::OptimizeAnalyzeResult;
use rspack_core::{
  render_init_fragments, AsyncDependenciesBlockIdentifier, BuildMetaExportsType, ChunkGraph,
  Compilation, DependenciesBlock, DependencyId, GenerateContext, Module, ModuleGraph, ParseContext,
  ParseResult, ParserAndGenerator, SideEffectsBailoutItem, SourceType, SpanExt, TemplateContext,
  TemplateReplaceSource,
};
use rspack_error::miette::Diagnostic;
use rspack_error::{DiagnosticExt, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_core::common::{Span, SyntaxContext};
use swc_core::ecma::parser::{EsConfig, Syntax};

use crate::dependency::HarmonyCompatibilityDependency;
use crate::inner_graph_plugin::InnerGraphPlugin;
use crate::visitors::ScanDependenciesResult;
use crate::visitors::{scan_dependencies, swc_visitor::resolver};
use crate::{SideEffectsFlagPluginVisitor, SyntaxContextInfo};

#[derive(Debug)]
pub struct JavaScriptParserAndGenerator;

#[allow(unused)]
impl JavaScriptParserAndGenerator {
  fn source_block(
    &self,
    compilation: &Compilation,
    block_id: &AsyncDependenciesBlockIdentifier,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let module_graph = compilation.get_module_graph();
    let block = module_graph
      .block_by_id(block_id)
      .expect("should have block");
    //    let block = block_id.expect_get(compilation);
    block.get_dependencies().iter().for_each(|dependency_id| {
      self.source_dependency(compilation, dependency_id, source, context)
    });
    block
      .get_blocks()
      .iter()
      .for_each(|block_id| self.source_block(compilation, block_id, source, context));
  }

  fn source_dependency(
    &self,
    compilation: &Compilation,
    dependency_id: &DependencyId,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    if let Some(dependency) = compilation
      .get_module_graph()
      .dependency_by_id(dependency_id)
      .expect("should have dependency")
      .as_dependency_template()
    {
      dependency.apply(source, context)
    }
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
      loaders,
      module_parser_options,
      ..
    } = parse_context;
    let mut diagnostics: Vec<Box<dyn Diagnostic + Send + Sync>> = vec![];

    let default_with_diagnostics = |diagnostics: Vec<Box<dyn Diagnostic + Send + Sync>>| {
      Ok(
        ParseResult {
          source: source.clone(),
          dependencies: vec![],
          blocks: vec![],
          presentational_dependencies: vec![],
          analyze_result: Default::default(),
          side_effects_bailout: None,
        }
        .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
          diagnostics,
          loaders,
        )),
      )
    };

    let (mut ast, fm) = match crate::ast::parse(
      source.source().to_string(),
      Syntax::Es(EsConfig {
        jsx: false,
        export_default_from: false,
        decorators: false,
        fn_bind: true,
        allow_super_outside_method: true,
        ..Default::default()
      }),
      &resource_data.resource_path.to_string_lossy(),
      module_type,
    ) {
      Ok(ast) => ast,
      Err(e) => {
        diagnostics.append(&mut e.into_iter().map(|e| e.boxed()).collect());
        return default_with_diagnostics(diagnostics);
      }
    };

    ast.transform(|program, context| {
      program.visit_mut_with(&mut resolver(
        context.unresolved_mark,
        context.top_level_mark,
        false,
      ));
    });

    let mut worker_syntax_list = WorkerSyntaxList::default();

    let ScanDependenciesResult {
      mut dependencies,
      blocks,
      presentational_dependencies,
      mut usage_span_record,
      import_map,
      mut warning_diagnostics,
    } = match ast.visit(|program, _| {
      scan_dependencies(
        &fm,
        program,
        &mut worker_syntax_list,
        resource_data,
        compiler_options,
        module_type,
        build_info,
        build_meta,
        module_identifier,
        module_parser_options,
      )
    }) {
      Ok(result) => result,
      Err(mut e) => {
        diagnostics.append(&mut e);
        return default_with_diagnostics(diagnostics);
      }
    };
    diagnostics.append(&mut warning_diagnostics);
    let mut side_effects_bailout = None;
    let analyze_result = if compiler_options.builtins.tree_shaking.enable() {
      let mut all_dependencies = dependencies.clone();
      for mut block in blocks.clone() {
        all_dependencies.extend(block.take_dependencies());
      }
      JsModule::new(
        &ast,
        &worker_syntax_list,
        &all_dependencies,
        module_identifier,
        compiler_options,
      )
      .analyze()
    } else {
      OptimizeAnalyzeResult::default()
    };

    if compiler_options.is_new_tree_shaking()
      && compiler_options.optimization.side_effects.is_true()
    {
      ast.transform(|program, context| {
        let unresolved_ctxt = SyntaxContext::empty().apply_mark(context.unresolved_mark);
        let mut visitor = SideEffectsFlagPluginVisitor::new(
          SyntaxContextInfo::new(unresolved_ctxt),
          program.comments.as_ref(),
        );
        program.visit_with(&mut visitor);
        build_meta.side_effect_free = Some(visitor.side_effects_item.is_none());
        // Take the item from visitor is safe, because the field is only used in this place
        side_effects_bailout = visitor
          .side_effects_item
          .take()
          .and_then(|item| -> Option<_> {
            let msg = span_to_location(item.span, &source.source())?;
            Some(SideEffectsBailoutItem { msg, ty: item.ty })
          })
      });
    }

    let inner_graph =
      if compiler_options.is_new_tree_shaking() && compiler_options.optimization.inner_graph {
        ast.transform(|program, context| {
          let unresolved_ctxt = SyntaxContext::empty().apply_mark(context.unresolved_mark);
          let top_level_ctxt = SyntaxContext::empty().apply_mark(context.top_level_mark);
          let mut plugin = InnerGraphPlugin::new(
            &mut dependencies,
            unresolved_ctxt,
            top_level_ctxt,
            &mut usage_span_record,
            &import_map,
            module_identifier,
            program.comments.take(),
          );
          plugin.enable();
          program.visit_with(&mut plugin);
          program.comments = plugin.comments.take();
          Some(plugin)
        })
      } else {
        None
      };

    if let Some(mut inner_graph) = inner_graph {
      inner_graph.infer_dependency_usage();
    }

    Ok(
      ParseResult {
        source,
        dependencies,
        blocks,
        presentational_dependencies,
        analyze_result,
        side_effects_bailout,
      }
      .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
        diagnostics,
        loaders,
      )),
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
        runtime: generate_context.runtime,
        concatenation_scope: generate_context.concatenation_scope.take(),
      };

      module.get_dependencies().iter().for_each(|dependency_id| {
        self.source_dependency(compilation, dependency_id, &mut source, &mut context)
      });

      if let Some(dependencies) = module.get_presentational_dependencies() {
        dependencies
          .iter()
          .for_each(|dependency| dependency.apply(&mut source, &mut context));
      };

      module
        .get_blocks()
        .iter()
        .for_each(|block_id| self.source_block(compilation, block_id, &mut source, &mut context));
      generate_context.concatenation_scope = context.concatenation_scope.take();
      render_init_fragments(source.boxed(), init_fragments, generate_context)
    } else {
      panic!(
        "Unsupported source type: {:?}",
        generate_context.requested_source_type
      )
    }
  }

  fn get_concatenation_bailout_reason(
    &self,
    module: &dyn rspack_core::Module,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<String> {
    // Only harmony modules are valid for optimization
    if module.build_meta().is_none()
      || module
        .build_meta()
        .map(|meta| meta.exports_type != BuildMetaExportsType::Namespace)
        .unwrap_or_default()
    {
      return Some(String::from("Module is not an ECMAScript module"));
    }

    if let Some(deps) = module.get_presentational_dependencies() {
      if !deps.iter().any(|dep| {
        // https://github.com/webpack/webpack/blob/b9fb99c63ca433b24233e0bbc9ce336b47872c08/lib/javascript/JavascriptGenerator.js#L65-L74
        dep
          .as_any()
          .downcast_ref::<HarmonyCompatibilityDependency>()
          .is_some()
      }) {
        return Some(String::from("Module is not an ECMAScript module"));
      }
    } else {
      return Some(String::from("Module is not an ECMAScript module"));
    }

    if let Some(info) = module.build_info()
      && let Some(bailout) = info.module_concatenation_bailout.as_deref()
    {
      return Some(format!("Module uses {bailout}",));
    }
    None
  }
}

fn span_to_location(span: Span, source: &str) -> Option<String> {
  let r = ropey::Rope::from_str(source);
  let start = span.real_lo();
  let end = span.real_hi();
  let start_char_offset = r.try_byte_to_char(start as usize).ok()?;
  let start_line = r.char_to_line(start_char_offset);
  let start_column = start_char_offset - r.line_to_char(start_line);

  let end_char_offset = r.try_byte_to_char(end as usize).ok()?;
  let end_line = r.char_to_line(end_char_offset);
  let end_column = end_char_offset - r.line_to_char(end_line);
  if start_line == end_line {
    Some(format!("{}:{start_column}-{end_column}", start_line + 1))
  } else {
    Some(format!(
      "{}:{start_column}-{}:{end_column}",
      start_line + 1,
      end_line + 1
    ))
  }
}

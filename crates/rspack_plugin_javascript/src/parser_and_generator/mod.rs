use std::sync::Arc;

use itertools::Itertools;
use rspack_core::diagnostics::map_box_diagnostics_to_module_parse_diagnostics;
use rspack_core::rspack_sources::{BoxSource, ReplaceSource, Source, SourceExt};
use rspack_core::{
  render_init_fragments, AsyncDependenciesBlockIdentifier, BuildMetaExportsType, ChunkGraph,
  Compilation, DependenciesBlock, DependencyId, GenerateContext, Module, ModuleGraph, ModuleType,
  ParseContext, ParseResult, ParserAndGenerator, SideEffectsBailoutItem, SourceType, SpanExt,
  TemplateContext, TemplateReplaceSource,
};
use rspack_error::miette::Diagnostic;
use rspack_error::{DiagnosticExt, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_core::common::comments::Comments;
use swc_core::common::input::SourceFileInput;
use swc_core::common::{FileName, Span, SyntaxContext};
use swc_core::ecma::ast;
use swc_core::ecma::parser::{lexer::Lexer, EsSyntax, Syntax};
use swc_node_comments::SwcComments;

use crate::dependency::HarmonyCompatibilityDependency;
use crate::visitors::{scan_dependencies, swc_visitor::resolver};
use crate::visitors::{semicolon, PathIgnoredSpans, ScanDependenciesResult};
use crate::{BoxJavascriptParserPlugin, SideEffectsFlagPluginVisitor, SyntaxContextInfo};

#[derive(Default)]
pub struct JavaScriptParserAndGenerator {
  parser_plugins: Vec<BoxJavascriptParserPlugin>,
}

impl std::fmt::Debug for JavaScriptParserAndGenerator {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JavaScriptParserAndGenerator")
      .field("parser_plugins", &"...")
      .finish()
  }
}

impl JavaScriptParserAndGenerator {
  pub fn add_parser_plugin(&mut self, parser_plugin: BoxJavascriptParserPlugin) {
    self.parser_plugins.push(parser_plugin);
  }

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

  fn size(&self, module: &dyn Module, _source_type: Option<&SourceType>) -> f64 {
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
      additional_data,
      ..
    } = parse_context;
    let mut diagnostics: Vec<Box<dyn Diagnostic + Send + Sync>> = vec![];

    let default_with_diagnostics =
      |source: Arc<dyn Source>, diagnostics: Vec<Box<dyn Diagnostic + Send + Sync>>| {
        Ok(
          ParseResult {
            source,
            dependencies: vec![],
            blocks: vec![],
            presentational_dependencies: vec![],
            code_generation_dependencies: vec![],
            side_effects_bailout: None,
          }
          .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
            diagnostics,
            loaders,
          )),
        )
      };

    let source = remove_bom(source);
    let cm: Arc<swc_core::common::SourceMap> = Default::default();
    let fm = cm.new_source_file(
      FileName::Custom(resource_data.resource_path.to_string_lossy().to_string()),
      source.source().to_string(),
    );
    let comments = SwcComments::default();
    let target = ast::EsVersion::EsNext;
    let lexer = Lexer::new(
      Syntax::Es(EsSyntax {
        allow_return_outside_function: matches!(
          module_type,
          ModuleType::JsDynamic | ModuleType::JsAuto
        ),
        ..Default::default()
      }),
      target,
      SourceFileInput::from(&*fm),
      Some(&comments),
    );

    let mut ast = match crate::ast::parse(
      lexer.clone(),
      &fm,
      cm.clone(),
      Some(comments.clone()),
      module_type,
    ) {
      Ok(ast) => ast,
      Err(e) => {
        diagnostics.append(&mut e.into_iter().map(|e| e.boxed()).collect());
        return default_with_diagnostics(source, diagnostics);
      }
    };

    let mut semicolons = Default::default();
    ast.transform(|program, context| {
      program.visit_mut_with(&mut resolver(
        context.unresolved_mark,
        context.top_level_mark,
        false,
      ));
      // dbg!(&resource_data.resource_path);
      // dbg!(lexer.clone().collect_vec());
      program.visit_with(&mut semicolon::InsertedSemicolons {
        semicolons: &mut semicolons,
        tokens: &lexer.collect_vec(),
      });
      // dbg!(&semicolons);
    });

    let mut path_ignored_spans = PathIgnoredSpans::default();

    let unresolved_mark = ast.get_context().unresolved_mark;

    let ScanDependenciesResult {
      dependencies,
      blocks,
      presentational_dependencies,
      mut warning_diagnostics,
      ..
    } = match ast.visit(|program, _| {
      scan_dependencies(
        cm.clone(),
        &fm,
        program,
        resource_data,
        compiler_options,
        module_type,
        build_info,
        build_meta,
        module_identifier,
        module_parser_options,
        &mut semicolons,
        &mut path_ignored_spans,
        unresolved_mark,
        &mut self.parser_plugins,
        additional_data,
      )
    }) {
      Ok(result) => result,
      Err(mut e) => {
        diagnostics.append(&mut e);
        return default_with_diagnostics(source, diagnostics);
      }
    };
    diagnostics.append(&mut warning_diagnostics);
    let mut side_effects_bailout = None;

    if compiler_options.optimization.side_effects.is_true() {
      ast.transform(|program, context| {
        let unresolved_ctxt = SyntaxContext::empty().apply_mark(context.unresolved_mark);
        let mut visitor = SideEffectsFlagPluginVisitor::new(
          SyntaxContextInfo::new(unresolved_ctxt),
          program.comments.as_ref().map(|c| c as &dyn Comments),
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

    // let inner_graph = if compiler_options.optimization.inner_graph {
    //   ast.transform(|program, context| {
    //     let unresolved_ctxt = SyntaxContext::empty().apply_mark(context.unresolved_mark);
    //     let top_level_ctxt = SyntaxContext::empty().apply_mark(context.top_level_mark);
    //     let mut plugin = InnerGraphPlugin::new(
    //       &mut dependencies,
    //       unresolved_ctxt,
    //       top_level_ctxt,
    //       &mut usage_span_record,
    //       &import_map,
    //       module_identifier,
    //       program.comments.take(),
    //       &path_ignored_spans,
    //     );
    //     plugin.enable();
    //     // program.visit_with(&mut plugin);
    //     program.comments = plugin.comments.take();
    //     Some(plugin)
    //   })
    // } else {
    //   None
    // };

    // if let Some(mut inner_graph) = inner_graph {
    //   inner_graph.infer_dependency_usage();
    // }

    Ok(
      ParseResult {
        source,
        dependencies,
        blocks,
        presentational_dependencies,
        code_generation_dependencies: vec![],
        side_effects_bailout,
      }
      .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
        diagnostics,
        loaders,
      )),
    )
  }

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
        data: generate_context.data,
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

fn remove_bom(s: Arc<dyn Source>) -> Arc<dyn Source> {
  if s.source().starts_with('\u{feff}') {
    let mut s = ReplaceSource::new(s);
    s.replace(0, 3, "", None);
    s.boxed()
  } else {
    s
  }
}

use std::borrow::Cow;
use std::sync::Arc;

use itertools::Itertools;
use rspack_cacheable::with::Skip;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::diagnostics::map_box_diagnostics_to_module_parse_diagnostics;
use rspack_core::rspack_sources::{BoxSource, ReplaceSource, Source, SourceExt};
use rspack_core::{
  remove_bom, render_init_fragments, AsyncDependenciesBlockIdentifier, BuildMetaExportsType,
  ChunkGraph, Compilation, DependenciesBlock, DependencyId, DependencyRange, GenerateContext,
  Module, ModuleGraph, ModuleType, ParseContext, ParseResult, ParserAndGenerator,
  SideEffectsBailoutItem, SourceType, TemplateContext, TemplateReplaceSource,
};
use rspack_error::miette::Diagnostic;
use rspack_error::{DiagnosticExt, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_core::common::comments::Comments;
use swc_core::common::input::SourceFileInput;
use swc_core::common::{FileName, SyntaxContext};
use swc_core::ecma::ast;
use swc_core::ecma::parser::{lexer::Lexer, EsSyntax, Syntax};
use swc_node_comments::SwcComments;

use crate::dependency::ESMCompatibilityDependency;
use crate::visitors::{scan_dependencies, swc_visitor::resolver};
use crate::visitors::{semicolon, ScanDependenciesResult};
use crate::{BoxJavascriptParserPlugin, SideEffectsFlagPluginVisitor, SyntaxContextInfo};

#[cacheable]
#[derive(Default)]
pub struct JavaScriptParserAndGenerator {
  // TODO
  #[cacheable(with=Skip)]
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

#[cacheable_dyn]
impl ParserAndGenerator for JavaScriptParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    SOURCE_TYPES
  }

  fn size(&self, module: &dyn Module, _source_type: Option<&SourceType>) -> f64 {
    module.source().map_or(0, |source| source.size()) as f64
  }

  #[tracing::instrument("JavaScriptParser:parse", skip_all)]
  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      module_type,
      module_layer,
      resource_data,
      compiler_options,
      build_info,
      build_meta,
      module_identifier,
      loaders,
      module_parser_options,
      additional_data,
      parse_meta,
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
      Arc::new(FileName::Custom(
        resource_data
          .resource_path
          .as_ref()
          .map(|p| p.as_str().to_string())
          .unwrap_or_default(),
      )),
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
        import_attributes: true,
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
      program.visit_with(&mut semicolon::InsertedSemicolons {
        semicolons: &mut semicolons,
        tokens: &lexer.collect_vec(),
      });
    });

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
        module_layer,
        build_info,
        build_meta,
        module_identifier,
        module_parser_options,
        &mut semicolons,
        unresolved_mark,
        &mut self.parser_plugins,
        additional_data,
        parse_meta,
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
            let source = source.source();
            let msg = Into::<DependencyRange>::into(item.span)
              .to_loc(Some(source.as_ref()))?
              .to_string();
            Some(SideEffectsBailoutItem { msg, ty: item.ty })
          })
      });
    }

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
  ) -> Option<Cow<'static, str>> {
    // Only ES modules are valid for optimization
    if module.build_meta().exports_type != BuildMetaExportsType::Namespace {
      return Some("Module is not an ECMAScript module".into());
    }

    if let Some(deps) = module.get_presentational_dependencies() {
      if !deps.iter().any(|dep| {
        // https://github.com/webpack/webpack/blob/b9fb99c63ca433b24233e0bbc9ce336b47872c08/lib/javascript/JavascriptGenerator.js#L65-L74
        dep
          .as_any()
          .downcast_ref::<ESMCompatibilityDependency>()
          .is_some()
      }) {
        return Some("Module is not an ECMAScript module".into());
      }
    } else {
      return Some("Module is not an ECMAScript module".into());
    }

    if let Some(bailout) = module.build_info().module_concatenation_bailout.as_deref() {
      return Some(format!("Module uses {bailout}").into());
    }
    None
  }
}

use std::{
  borrow::Cow,
  sync::{Arc, LazyLock},
};

use regex::Regex;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsInner, Skip},
};
use rspack_core::{
  ArcComputed, AsyncDependenciesBlockIdentifier, BuildMetaExportsType,
  COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY, ChunkGraph, CollectedTypeScriptInfo, Compilation,
  DependenciesBlock, DependencyId, GenerateContext, ImportMeta, Module, ModuleArgument,
  ModuleCodeTemplate, ModuleGraph, ModuleType, ParseContext, ParseResult, ParserAndGenerator,
  ResolvedModuleOptions, RuntimeGlobals, RuntimeGlobalsRenderMode, RuntimeVariable,
  SideEffectsBailoutItem, SourceType, TemplateContext, TemplateReplaceSource,
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics,
  remove_bom, render_init_fragments,
  rspack_sources::{BoxSource, ReplaceSource, Source, SourceExt},
};
use rspack_error::{
  Diagnostic, Error, IntoTWithDiagnosticArray, Result, Severity as RspackSeverity,
  TWithDiagnosticArray,
};
use rspack_util::swc::RspackComments;
use rustc_hash::FxHashSet;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, Severity as SwcSeverity, SourceType as SwcSourceType, VisitWith};
use swc_next_ecma_parser::{CommentMode, Options, ParseReturn, Parser, TokenParserConfig};
use swc_next_ecma_semantic::{AnalyzeOptions, SemanticReturn, analyze};

use crate::{
  BoxJavascriptParserPlugin,
  dependency::ESMCompatibilityDependency,
  visitors::{ParsedJavaScriptAst, ScanDependenciesResult, scan_dependencies, semicolon},
};

#[derive(Debug)]
pub struct ParserRuntimeRequirementsData {
  pub render_mode: RuntimeGlobalsRenderMode,
  pub context: String,
  pub module: String,
  pub rspack_module: String,
  pub exports: String,
  pub require: String,
  pub compatibility_runtime_scope: String,
  pub require_regex: &'static LazyLock<Regex>,
  pub module_cache: String,
  pub entry_module_id: String,
}

static LEGACY_REQUIRE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new("__webpack_require__\\s*(!?\\.)").expect("should init `REQUIRE_FUNCTION_REGEX`")
});

// SWC Next diagnostics currently expose severity, span, and message, but no
// stable diagnostic kind. Keep message-based legacy compatibility confined to
// these predicates so parser control flow does not depend on strings elsewhere.
fn is_top_level_return_diagnostic(diagnostic: &swc_next_ecma_ast::Diagnostic<'_>) -> bool {
  diagnostic.message == "'return' statement is only valid inside a function"
}

fn is_stricter_than_legacy_semantic_diagnostic(
  diagnostic: &swc_next_ecma_ast::Diagnostic<'_>,
) -> bool {
  let message = diagnostic.message.as_ref();
  (message.starts_with("Identifier '") && message.ends_with("' has already been declared"))
    // Rspack intentionally keeps regexp pattern/flags raw and lets consumers
    // such as import.meta.webpackContext downgrade invalid regexps to warnings.
    || message == "Invalid regular expression literal"
}

fn append_swc_next_diagnostics<'a>(
  diagnostics: &mut Vec<Diagnostic>,
  source: &str,
  errors: impl IntoIterator<Item = swc_next_ecma_ast::Diagnostic<'a>>,
) {
  let mut visited = FxHashSet::default();
  let mut shared_source = None;
  diagnostics.extend(errors.into_iter().filter_map(|diagnostic| {
    let span = diagnostic.span;
    if !visited.insert((span.start, span.end)) {
      return None;
    }
    let source = shared_source.get_or_insert_with(|| Arc::<str>::from(source));
    let mut error = Error::from_shared_source(
      Some(source.clone()),
      span.start as usize,
      span.end as usize,
      "JavaScript parse error".to_string(),
      diagnostic.message.into_owned(),
    );
    error.severity = match diagnostic.severity {
      SwcSeverity::Error => RspackSeverity::Error,
      SwcSeverity::Warning => RspackSeverity::Warning,
      SwcSeverity::Hint | SwcSeverity::Info => RspackSeverity::Warning,
    };
    Some(error.into())
  }));
}

impl ParserRuntimeRequirementsData {
  pub fn new(runtime_template: &ModuleCodeTemplate) -> Self {
    let require_name =
      runtime_template.render_runtime_globals_without_adding(&RuntimeGlobals::REQUIRE);
    let compatibility_runtime_scope = runtime_template.render_runtime_scope();
    let module_name =
      runtime_template.render_runtime_globals_without_adding(&RuntimeGlobals::MODULE);
    let exports_name =
      runtime_template.render_runtime_globals_without_adding(&RuntimeGlobals::EXPORTS);
    let module_cache_name =
      runtime_template.render_runtime_globals_without_adding(&RuntimeGlobals::MODULE_CACHE);
    let entry_module_id_name =
      runtime_template.render_runtime_globals_without_adding(&RuntimeGlobals::ENTRY_MODULE_ID);
    let context_name = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
    let rspack_module_name = runtime_template.render_runtime_variable(&RuntimeVariable::Module);
    Self {
      render_mode: runtime_template.render_mode(),
      require_regex: &LEGACY_REQUIRE_REGEX,
      context: context_name,
      module: module_name,
      rspack_module: rspack_module_name,
      exports: exports_name,
      require: require_name,
      compatibility_runtime_scope,
      module_cache: module_cache_name,
      entry_module_id: entry_module_id_name,
    }
  }

  pub fn module_argument(&self, module_argument: &ModuleArgument) -> String {
    match module_argument {
      ModuleArgument::Module => self.module.clone(),
      ModuleArgument::RspackModule => self.rspack_module.clone(),
    }
  }
}

#[cacheable]
pub struct JavaScriptParserAndGenerator {
  #[cacheable(with=AsInner)]
  import_meta: ArcComputed<ResolvedModuleOptions, ImportMeta>,
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
  pub fn new(module_options: Arc<ResolvedModuleOptions>) -> Self {
    Self {
      import_meta: ArcComputed::new(module_options, |options| options.into()),
      parser_plugins: Vec::default(),
    }
  }

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
      .as_dependency_code_generation()
    {
      if let Some(template) = dependency
        .dependency_template()
        .and_then(|template_type| compilation.get_dependency_template(template_type))
      {
        template.render(dependency, source, context)
      } else {
        panic!(
          "Can not find dependency template of {:?}",
          dependency.dependency_template()
        );
      }
    }
  }
}

static SOURCE_TYPES: &[SourceType; 1] = &[SourceType::JavaScript];

#[cacheable_dyn]
#[async_trait::async_trait]
impl ParserAndGenerator for JavaScriptParserAndGenerator {
  fn source_types(&self, _module: &dyn Module, _module_graph: &ModuleGraph) -> &[SourceType] {
    SOURCE_TYPES
  }

  fn size(&self, module: &dyn Module, _source_type: Option<&SourceType>) -> f64 {
    module.source().map_or(0, |source| source.size()) as f64
  }

  #[tracing::instrument("JavaScriptParser:parse", skip_all,fields(
    resource = parse_context.resource_data.resource()
  ))]
  async fn parse<'a>(
    &mut self,
    parse_context: ParseContext<'a>,
  ) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      module_type,
      module_layer,
      resource_data,
      compiler_options,
      runtime_template,
      factory_meta,
      build_info,
      build_meta,
      module_identifier,
      loaders,
      module_parser_options,
      mut parse_meta,
      ..
    } = parse_context;
    let mut diagnostics: Vec<Diagnostic> = vec![];

    if let Some(collected_ts_info) = parse_meta.remove(COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY)
      && let Ok(collected_ts_info) = collected_ts_info
        .into_any()
        .downcast::<CollectedTypeScriptInfo>()
    {
      build_info.collected_typescript_info = Some(*collected_ts_info);
    }

    let default_with_diagnostics = |source: Arc<dyn Source>, diagnostics: Vec<Diagnostic>| {
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
    let source_string = source.source().into_string_lossy();

    let jsx = module_parser_options
      .and_then(|options| options.get_javascript())
      .and_then(|options| options.jsx)
      .unwrap_or(false);

    let allocator = Allocator::new();
    let source_type = match module_type {
      ModuleType::JsEsm => SwcSourceType::Module,
      ModuleType::JsDynamic => SwcSourceType::CommonJs,
      _ => SwcSourceType::Unambiguous,
    };
    let parse_with_source_type = |source_type| {
      Parser::init(
        &allocator,
        source_string.as_ref(),
        Options {
          source_type,
          lang: if jsx { Lang::Jsx } else { Lang::Js },
          preserve_parens: false,
          comments: CommentMode::Flat,
        },
        TokenParserConfig,
      )
      .parse()
    };
    let mut parse_return = parse_with_source_type(source_type);
    // The legacy `JsAuto` parser allowed a CommonJS-style top-level return.
    // SWC Next's unambiguous mode resolves non-ESM input as `Script`, where
    // return is rejected. Retry only this compatibility case as CommonJS;
    // other parse errors and files containing ESM syntax remain diagnostics.
    if module_type.is_js_auto()
      && parse_return
        .diagnostics
        .iter()
        .any(is_top_level_return_diagnostic)
    {
      parse_return = parse_with_source_type(SwcSourceType::CommonJs);
    }
    let ParseReturn {
      ast,
      tokens,
      diagnostics: parse_diagnostics,
    } = parse_return;

    if !parse_diagnostics.is_empty() {
      append_swc_next_diagnostics(&mut diagnostics, &source_string, parse_diagnostics);
      return default_with_diagnostics(source, diagnostics);
    }

    let SemanticReturn {
      semantic,
      diagnostics: mut semantic_diagnostics,
    } = analyze(
      &ast,
      AnalyzeOptions {
        check_syntax: true,
        build_module_record: false,
      },
    );
    // The legacy parser accepted several redeclaration combinations that the
    // SWC Next semantic checker rejects. The parser has already emitted its
    // own early errors, so exclude only known stricter-than-legacy categories.
    semantic_diagnostics
      .retain(|diagnostic| !is_stricter_than_legacy_semantic_diagnostic(diagnostic));
    if !semantic_diagnostics.is_empty() {
      append_swc_next_diagnostics(&mut diagnostics, &source_string, semantic_diagnostics);
      return default_with_diagnostics(source, diagnostics);
    }

    let program = ast.root_program();
    let comments = RspackComments::from_ast(&ast);
    let mut semicolons = Default::default();
    program.visit_with(&mut semicolon::InsertedSemicolons::new(
      &ast,
      &mut semicolons,
      &tokens,
    ));
    let parsed_ast = ParsedJavaScriptAst {
      ast: &ast,
      comments: &comments,
      semantic: &semantic,
      program,
    };
    let parser_runtime_requirements = ParserRuntimeRequirementsData::new(runtime_template);

    let ScanDependenciesResult {
      dependencies,
      blocks,
      presentational_dependencies,
      mut warning_diagnostics,
      mut side_effects_item,
    } = match scan_dependencies(
      &source_string,
      &parsed_ast,
      resource_data,
      compiler_options,
      module_type,
      module_layer,
      factory_meta,
      build_meta,
      build_info,
      module_identifier,
      module_parser_options,
      ArcComputed::clone(&self.import_meta),
      &mut semicolons,
      &mut self.parser_plugins,
      parse_meta,
      &parser_runtime_requirements,
    ) {
      Ok(result) => result,
      Err(mut e) => {
        diagnostics.append(&mut e);
        return default_with_diagnostics(source, diagnostics);
      }
    };
    diagnostics.append(&mut warning_diagnostics);
    let mut side_effects_bailout = None;

    if compiler_options.optimization.side_effects.is_true() {
      let has_side_effects = side_effects_item.is_some();
      build_meta.set_side_effect_free(!has_side_effects);
      if has_side_effects {
        build_info.deferred_pure_checks.clear();
      }
      side_effects_bailout = side_effects_item.take().and_then(|item| -> Option<_> {
        let msg = item.loc?.to_string();
        Some(SideEffectsBailoutItem { msg, ty: item.ty })
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

  async fn generate(
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
        init_fragments: &mut init_fragments,
        runtime: generate_context.runtime,
        concatenation_scope: generate_context.concatenation_scope.take(),
        data: generate_context.data,
        runtime_template: generate_context.runtime_template,
      };

      module.get_dependencies().iter().for_each(|dependency_id| {
        self.source_dependency(compilation, dependency_id, &mut source, &mut context)
      });

      if let Some(dependencies) = module.get_presentational_dependencies() {
        dependencies.iter().for_each(|dependency| {
          if let Some(template) = dependency
            .dependency_template()
            .and_then(|template_type| compilation.get_dependency_template(template_type))
          {
            template.render(dependency.as_ref(), &mut source, &mut context)
          } else {
            panic!(
              "Can not find dependency template of {:?}",
              dependency.dependency_template()
            );
          }
        });
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
    if module.build_meta().exports_type() != BuildMetaExportsType::Namespace {
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

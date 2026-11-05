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
  ArcComputed, AsyncDependenciesBlockIdentifier, BuildInfo, BuildMeta, BuildMetaExportsType,
  COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY, ChunkGraph, CollectedTypeScriptInfo, Compilation,
  CompilerOptions, DependenciesBlock, Dependency, DependencyCodeGenerationRef, FactoryMeta,
  GenerateContext, ImportMeta, Module, ModuleArgument, ModuleCodeTemplate, ModuleGraph,
  ModuleIdentifier, ModuleLayer, ModuleType, ParseContext, ParseMeta, ParseResult,
  ParserAndGenerator, ParserOptions, ResolvedModuleOptions, ResourceData, RuntimeGlobals,
  RuntimeGlobalsRenderMode, RuntimeVariable, SideEffectsBailoutItem, SourceType, TemplateContext,
  TemplateReplaceSource,
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
use swc_next_ecma_ast::{
  Ast, Diagnostic as SwcDiagnostic, GetSpan, Lang, NodeId, NodeKindData, Severity as SwcSeverity,
  SourceType as SwcSourceType, Span,
};
use swc_next_ecma_parser::{
  CommentMode, Options, ParseReturn, Parser, Token, TokenParserConfig, TokenTag,
};
use swc_next_ecma_semantic::{AnalyzeOptions, SemanticReturn, SyntaxChecks, analyze};

use crate::{
  BoxJavascriptParserPlugin,
  dependency::ESMCompatibilityDependency,
  portable_paths::{self, Canonical, Prepared},
  visitors::{ParsedJavaScriptAst, ScanDependenciesResult, scan_dependencies},
};

#[derive(Debug)]
pub struct ParserRuntimeRequirementsData {
  pub render_mode: RuntimeGlobalsRenderMode,
  pub context: &'static str,
  pub module: Arc<str>,
  pub rspack_module: &'static str,
  pub exports: Arc<str>,
  pub require: Arc<str>,
  pub compatibility_runtime_scope: Arc<str>,
  pub require_regex: &'static LazyLock<Regex>,
  pub module_cache: Arc<str>,
  pub entry_module_id: Arc<str>,
}

static LEGACY_REQUIRE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new("__webpack_require__\\s*(!?\\.)").expect("should init `REQUIRE_FUNCTION_REGEX`")
});

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
      ModuleArgument::Module => self.module.to_string(),
      ModuleArgument::RspackModule => self.rspack_module.to_string(),
    }
  }
}

/// Module inputs shared by every parse of one build.
struct ParseInput<'a> {
  module_type: &'a ModuleType,
  module_layer: Option<&'a ModuleLayer>,
  resource_data: &'a ResourceData,
  compiler_options: &'a CompilerOptions,
  runtime_template: &'a ModuleCodeTemplate,
  factory_meta: Option<&'a FactoryMeta>,
  module_identifier: ModuleIdentifier,
  module_parser_options: Option<&'a ParserOptions>,
  source_type: SwcSourceType,
  lang: Lang,
}

/// One parsed and scanned text, before diagnostics are attributed to loaders.
struct ParsedText {
  result: ParseResult,
  diagnostics: Vec<Diagnostic>,
  portable_site_evaluated: bool,
  portable_site_cut: bool,
}

fn clone_parse_meta(parse_meta: &ParseMeta) -> ParseMeta {
  parse_meta
    .iter()
    .map(|(key, value)| (key.clone(), value.clone_parse_meta()))
    .collect()
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
  pub(crate) fn concatenation_bailout_reason(
    &self,
    module: &dyn rspack_core::Module,
    allow_empty_commonjs: bool,
  ) -> Option<Cow<'static, str>> {
    // Only ESM eligibility may be skipped for a validated empty CommonJS candidate.
    // All other bailouts stay on the ordinary concatenation path.
    if !allow_empty_commonjs {
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
    }

    if let Some(bailout) = module.build_info().module_concatenation_bailout.as_deref() {
      return Some(format!("Module uses {bailout}").into());
    }
    None
  }

  pub fn new(module_options: Arc<ResolvedModuleOptions>) -> Self {
    Self {
      import_meta: ArcComputed::new(module_options, |options| options.into()),
      parser_plugins: Vec::default(),
    }
  }

  pub fn add_parser_plugin(&mut self, parser_plugin: BoxJavascriptParserPlugin) {
    self.parser_plugins.push(parser_plugin);
  }

  /// Parses and scans one text. With `canonical`, `source` is the canonical text and its site
  /// literals get their real values before any analysis. Returns `None` when the canonical
  /// text does not hold one string literal at each site.
  #[allow(clippy::too_many_arguments)]
  fn parse_text(
    &mut self,
    input: &ParseInput<'_>,
    source: BoxSource,
    canonical: Option<&Canonical>,
    build_info: &mut BuildInfo,
    build_meta: &mut BuildMeta,
    parse_meta: ParseMeta,
  ) -> Option<ParsedText> {
    let mut diagnostics: Vec<Diagnostic> = vec![];
    let unparsed = |source: BoxSource, diagnostics: Vec<Diagnostic>| {
      Some(ParsedText {
        result: ParseResult {
          source,
          dependencies: vec![],
          blocks: vec![],
          presentational_dependencies: vec![],
          code_generation_dependencies: vec![],
          side_effects_bailout: None,
        },
        diagnostics,
        portable_site_evaluated: false,
        portable_site_cut: false,
      })
    };

    let source_string = source.source().into_string_lossy();
    let allocator = Allocator::new();
    let ParseReturn {
      mut ast,
      tokens,
      diagnostics: parse_diagnostics,
    } = Parser::init(
      &allocator,
      source_string.as_ref(),
      Options {
        source_type: input.source_type,
        lang: input.lang,
        preserve_parens: false,
        comments: CommentMode::Flat,
      },
      TokenParserConfig,
    )
    .parse();

    if !parse_diagnostics.is_empty() {
      append_swc_next_diagnostics(&mut diagnostics, &source_string, parse_diagnostics);
      return unparsed(source, diagnostics);
    }

    let portable_sites = match canonical {
      Some(canonical) => {
        if !canonical.restore_real_literals(&mut ast) {
          return None;
        }
        Some(canonical.site_ranges())
      }
      None => None,
    };

    let mut semicolons = Default::default();
    let scan_diagnostics = AstScanner::new(&mut semicolons, &tokens).scan(&ast);
    if !scan_diagnostics.is_empty() {
      append_swc_next_diagnostics(&mut diagnostics, &source_string, scan_diagnostics);
      return unparsed(source, diagnostics);
    }

    // Keep the legacy SWC parser's permissive behavior for these semantic checks.
    let check_syntax = SyntaxChecks::all()
      - SyntaxChecks::REDECLARATIONS
      - SyntaxChecks::TS_IMPORT_ALIAS_CONFLICTS
      - SyntaxChecks::REGEXP_LITERALS
      - SyntaxChecks::PRIVATE_NAME_REFERENCES
      - SyntaxChecks::DUPLICATE_PRIVATE_NAMES
      - SyntaxChecks::DELETE_PRIVATE_FIELDS
      - SyntaxChecks::SUPER_CALLS
      - SyntaxChecks::FUNCTION_DECLARATIONS
      - SyntaxChecks::CLASS_MEMBER_NAMES
      - SyntaxChecks::STRICT_BINDING_NAMES
      - SyntaxChecks::STRICT_ASSIGNMENT_TARGETS
      - SyntaxChecks::DUPLICATE_EXPORTS
      - SyntaxChecks::UNRESOLVED_EXPORTS
      - SyntaxChecks::CONTINUE_STATEMENTS
      - SyntaxChecks::BREAK_STATEMENTS
      - SyntaxChecks::AWAIT_EXPRESSIONS
      - SyntaxChecks::YIELD_EXPRESSIONS;
    let SemanticReturn {
      semantic,
      diagnostics: semantic_diagnostics,
    } = analyze(&ast, AnalyzeOptions { check_syntax });
    if !semantic_diagnostics.is_empty() {
      append_swc_next_diagnostics(&mut diagnostics, &source_string, semantic_diagnostics);
      return unparsed(source, diagnostics);
    }

    let program = ast.root_program();
    let comments = RspackComments::from_ast(&ast);
    let parsed_ast = ParsedJavaScriptAst {
      ast: &ast,
      comments: &comments,
      semantic: &semantic,
      program,
    };
    let parser_runtime_requirements = ParserRuntimeRequirementsData::new(input.runtime_template);
    let portable_location_shifts =
      canonical.map(|canonical| canonical.location_shifts(&source_string));

    let ScanDependenciesResult {
      dependencies,
      blocks,
      presentational_dependencies,
      mut warning_diagnostics,
      mut side_effects_item,
      portable_site_evaluated,
      portable_site_cut,
    } = match scan_dependencies(
      &source_string,
      &parsed_ast,
      input.resource_data,
      input.compiler_options,
      input.module_type,
      input.module_layer,
      input.factory_meta,
      build_meta,
      build_info,
      input.module_identifier,
      input.module_parser_options,
      ArcComputed::clone(&self.import_meta),
      &mut semicolons,
      &mut self.parser_plugins,
      parse_meta,
      &parser_runtime_requirements,
      portable_sites.as_deref(),
      portable_location_shifts.as_deref(),
    ) {
      Ok(result) => result,
      Err(mut e) => {
        diagnostics.append(&mut e);
        return unparsed(source, diagnostics);
      }
    };
    diagnostics.append(&mut warning_diagnostics);
    let mut side_effects_bailout = None;

    if input.compiler_options.optimization.side_effects.is_true() {
      let has_side_effects = side_effects_item.is_some();
      build_meta.set_side_effect_free(!has_side_effects);
      if has_side_effects {
        build_info.deferred_pure_checks.clear();
      }
      side_effects_bailout = side_effects_item.take().and_then(|item| -> Option<_> {
        Some(SideEffectsBailoutItem {
          loc: item.loc?,
          ty: item.ty,
        })
      });
    }

    Some(ParsedText {
      result: ParseResult {
        source,
        dependencies,
        blocks,
        presentational_dependencies,
        code_generation_dependencies: vec![],
        side_effects_bailout,
      },
      diagnostics,
      portable_site_evaluated,
      portable_site_cut,
    })
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
    block.get_dependencies().iter().for_each(|dependency| {
      self.source_dependency(compilation, dependency.as_ref(), source, context)
    });
    block
      .get_blocks()
      .iter()
      .for_each(|block_id| self.source_block(compilation, block_id, source, context));
  }

  fn source_dependency(
    &self,
    compilation: &Compilation,
    dependency: &dyn Dependency,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    if let Some(dependency) = dependency.as_dependency_code_generation() {
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
    // The stored source, so a restored module and a cold build in another root agree.
    let source = match module.as_normal_module() {
      Some(module) => module.normalized_source(),
      None => module.source(),
    };
    source.map_or(0, |source| source.size()) as f64
  }

  fn source_view(
    &self,
    source: &BoxSource,
    presentational_dependencies: &[DependencyCodeGenerationRef],
  ) -> Option<rspack_core::SourceView> {
    portable_paths::render_view(source, presentational_dependencies)
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
      module_context,
      mut parse_meta,
      portable_output,
      portable_requests,
      ..
    } = parse_context;

    if let Some(collected_ts_info) = parse_meta.remove(COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY)
      && let Ok(collected_ts_info) = collected_ts_info
        .into_any()
        .downcast::<CollectedTypeScriptInfo>()
    {
      build_info.collected_typescript_info = Some(*collected_ts_info);
    }

    let mut source = remove_bom(source);

    let jsx = module_parser_options
      .and_then(|options| options.get_javascript())
      .and_then(|options| options.jsx)
      .unwrap_or(false);
    let source_type = match module_type {
      ModuleType::JsEsm => SwcSourceType::Module,
      ModuleType::JsDynamic => SwcSourceType::CommonJs,
      _ => SwcSourceType::Unambiguous,
    };
    let lang = if jsx { Lang::Jsx } else { Lang::Js };
    let input = ParseInput {
      module_type,
      module_layer,
      resource_data,
      compiler_options,
      runtime_template,
      factory_meta,
      module_identifier,
      module_parser_options,
      source_type,
      lang,
    };

    let create_require = module_parser_options
      .and_then(|options| options.get_javascript())
      .is_some_and(|options| options.is_create_require_enabled());

    // A request a loader generated for this checkout is spelled relative to the module, so the
    // dependency, source, map and hashes hold no root. Only the scan can prove what is a request.
    // createRequire resolves against its own base, so a relative spelling could change the target.
    if let Some(requests) = portable_requests.filter(|_| !create_require) {
      let saved_build_info = build_info.clone();
      let saved_build_meta = build_meta.clone();
      let scanned = self.parse_text(
        &input,
        source.clone(),
        Some(&Canonical::unchanged(source.clone())),
        build_info,
        build_meta,
        clone_parse_meta(&parse_meta),
      );
      *build_info = saved_build_info;
      *build_meta = saved_build_meta;
      if let Some(relative) = scanned
        .filter(|scanned| scanned.diagnostics.is_empty())
        .and_then(|scanned| {
          portable_paths::relativize_requests(
            &source,
            &scanned.result,
            module_context.as_path(),
            requests.root,
            source_type,
            lang,
          )
        })
      {
        if requests.rebase {
          source = relative;
        } else {
          build_info.absolute_loader_requests = true;
        }
      }
    }

    // Certified literals are normalized with or without a persistent cache, so cache:false and
    // cached builds emit the same code, maps and hashes.
    let mut root_sensitive = false;
    if let Some(paths) = portable_output.filter(|paths| !paths.is_empty()) {
      let text = source.source().into_string_lossy();
      let prepared = if create_require {
        // createRequire replacements copy source spans, including enclosing expressions.
        // Those spans must contain real paths rather than canonical placeholders.
        Prepared::Unsupported
      } else {
        portable_paths::prepare(
          &source,
          &text,
          paths,
          compiler_options.context.as_path(),
          source_type,
          lang,
        )
      };
      match prepared {
        Prepared::Unchanged => {}
        Prepared::Unsupported => root_sensitive = true,
        Prepared::Canonical(canonical) => {
          let saved_build_info = build_info.clone();
          let saved_build_meta = build_meta.clone();
          let saved_parse_meta = clone_parse_meta(&parse_meta);
          let parsed = self.parse_text(
            &input,
            canonical.source.clone(),
            Some(&canonical),
            build_info,
            build_meta,
            parse_meta,
          );
          match parsed {
            // Diagnostics excerpt the source text, and a range that cuts into a literal was
            // computed from canonical bytes, so either keeps the real text.
            Some(ParsedText {
              mut result,
              diagnostics,
              portable_site_evaluated,
              portable_site_cut: false,
            }) if diagnostics.is_empty() => {
              // Match the ordinary scan, which consumes the module's parser plugins.
              self.parser_plugins.clear();
              build_info.root_sensitive |= portable_site_evaluated;
              result
                .presentational_dependencies
                .extend(canonical.into_dependencies());
              return Ok(result.with_empty_diagnostic());
            }
            _ => {
              *build_info = saved_build_info;
              *build_meta = saved_build_meta;
              parse_meta = saved_parse_meta;
              root_sensitive = true;
            }
          }
        }
      }
    }

    let ParsedText {
      result,
      diagnostics,
      ..
    } = self
      .parse_text(&input, source, None, build_info, build_meta, parse_meta)
      .expect("the real text has no canonical literals to restore");
    if root_sensitive {
      // The module holds a certified path that is not relocated, possibly without root bytes.
      build_info.root_sensitive = true;
    }
    Ok(
      result.with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
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

      module.get_dependencies().iter().for_each(|dependency| {
        self.source_dependency(compilation, dependency.as_ref(), &mut source, &mut context)
      });

      let mut portable_sites = vec![];
      if let Some(dependencies) = module.get_presentational_dependencies() {
        dependencies.iter().for_each(|dependency| {
          if let Some(site) = portable_paths::as_site(dependency) {
            portable_sites.push(site);
            return;
          }
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
      // Certified literals render last, so they see every replacement that removed them.
      for site in portable_sites {
        site.render(&mut source);
      }
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
    self.concatenation_bailout_reason(module, false)
  }
}

/// Collects inserted semicolon positions and rejects syntax unsupported by the
/// legacy parser in a single pass over the flat AST.
pub struct AstScanner<'view> {
  semicolons: &'view mut FxHashSet<u32>,
  tokens: &'view [Token],
}

impl<'view> AstScanner<'view> {
  pub fn new(semicolons: &'view mut FxHashSet<u32>, tokens: &'view [Token]) -> Self {
    Self { semicolons, tokens }
  }

  #[inline]
  fn curr_token(&self, span: Span) -> Option<usize> {
    self
      .tokens
      .binary_search_by(|token| token.span().start.cmp(&span.start))
      .ok()
  }

  #[inline]
  fn next_token(&self, span: Span) -> Option<usize> {
    self
      .tokens
      .binary_search_by(|token| token.span().end.cmp(&span.end))
      .ok()
      .map(|index| index + 1)
  }

  #[inline]
  fn can_insert_semi(&self, token_index: usize) -> bool {
    if token_index == self.tokens.len() {
      return true;
    }
    let token = self.tokens[token_index];
    token.tag() == TokenTag::RightBrace || token.has_line_terminator_before()
  }

  #[inline]
  fn semi(&mut self, span: Span) {
    let Some(index) = self.curr_token(span) else {
      return;
    };
    if index > 0 {
      let previous = self.tokens[index - 1];
      if previous.tag() != TokenTag::Semicolon && self.can_insert_semi(index) {
        self.semicolons.insert(previous.span().end);
      }
    }
  }

  #[inline]
  fn post_semi(&mut self, span: Span) {
    let Some(index) = self.next_token(span) else {
      return;
    };
    if index > 0 {
      let previous = self.tokens[index - 1];
      if previous.tag() != TokenTag::Semicolon && self.can_insert_semi(index) {
        self.semicolons.insert(previous.span().end);
      }
    }
  }

  /// Requires successful parser output before transformations can detach nodes.
  pub fn scan(&mut self, ast: &Ast<'_>) -> Vec<SwcDiagnostic<'static>> {
    let mut diagnostics = Vec::new();
    for index in 0..ast.node_count() {
      let node = NodeId::from_raw_unchecked(index as u32);
      match ast.kind_data(node) {
        NodeKindData::Decorator(_) => {
          let start = ast.span(node).start;
          diagnostics.push(SwcDiagnostic::error(
            "Decorators are not supported",
            Span::new(start, start + 1),
          ));
        }
        NodeKindData::PropertyDefinition(property) => {
          if property.accessor(ast) {
            diagnostics.push(SwcDiagnostic::error(
              "Auto-accessors are not supported",
              property.key(ast).span(ast),
            ));
          }
          self.post_semi(ast.span(node));
        }
        NodeKindData::UpdateExpression(_) => self.semi(ast.span(node)),
        NodeKindData::ExpressionStatement(_)
        | NodeKindData::VariableDeclaration(_)
        | NodeKindData::ContinueStatement(_)
        | NodeKindData::BreakStatement(_)
        | NodeKindData::ReturnStatement(_)
        | NodeKindData::ThrowStatement(_)
        | NodeKindData::YieldExpression(_)
        | NodeKindData::ImportDeclaration(_)
        | NodeKindData::ExportNamedDeclaration(_)
        | NodeKindData::ExportDefaultDeclaration(_)
        | NodeKindData::ExportAllDeclaration(_)
        | NodeKindData::DebuggerStatement(_) => self.post_semi(ast.span(node)),
        _ => {}
      }
    }
    diagnostics
  }
}

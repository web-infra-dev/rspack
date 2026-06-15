use std::{
  borrow::Cow,
  collections::HashSet,
  sync::{Arc, LazyLock},
};

use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsyncDependenciesBlockIdentifier, BuildMetaExportsType, COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY,
  ChunkGraph, CollectedTypeScriptInfo, Compilation, DependenciesBlock, DependencyId,
  GenerateContext, Module, ModuleArgument, ModuleCodeTemplate, ModuleGraph, ModuleType,
  ParseContext, ParseResult, ParserAndGenerator, RuntimeGlobals, RuntimeVariable,
  SideEffectsBailoutItem, SourceType, TemplateContext, TemplateReplaceSource,
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics,
  remove_bom, render_init_fragments,
  rspack_sources::{BoxSource, ReplaceSource, Source, SourceExt},
};
use rspack_error::{Diagnostic, Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Comments, EsVersion, Program, VisitWith};
use swc_experimental_ecma_parser::{
  EsSyntax, Lexer, Parser, StringSource, Syntax, unstable::Capturing,
};
use swc_experimental_ecma_semantic::resolver::resolver;
use swc_experimental_ecma_transforms_base::remove_paren::remove_paren;

use crate::{
  BoxJavascriptParserPlugin,
  dependency::ESMCompatibilityDependency,
  visitors::{ParsedJavaScriptAst, ScanDependenciesResult, scan_dependencies, semicolon},
};

#[derive(Debug)]
pub struct ParserRuntimeRequirementsData {
  pub context: String,
  pub module: String,
  pub rspack_module: String,
  pub exports: String,
  pub require: String,
  pub require_regex: &'static LazyLock<Regex>,
  pub module_cache: String,
  pub entry_module_id: String,
}

static LEGACY_REQUIRE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new("__webpack_require__\\s*(!?\\.)").expect("should init `REQUIRE_FUNCTION_REGEX`")
});

fn append_experimental_parse_errors(
  diagnostics: &mut Vec<Diagnostic>,
  source: &str,
  errors: impl IntoIterator<Item = swc_experimental_ecma_parser::error::Error>,
) {
  let mut visited = HashSet::new();
  diagnostics.extend(errors.into_iter().filter_map(|err| {
    let span = err.span();
    let message = err.kind().msg().to_string();
    if !visited.insert((message.clone(), span)) {
      return None;
    }
    Some(
      Error::from_string(
        Some(source.to_string()),
        span.start.saturating_sub(1) as usize,
        span.end.saturating_sub(1) as usize,
        "JavaScript parse error".to_string(),
        message,
      )
      .into(),
    )
  }));
}

impl ParserRuntimeRequirementsData {
  pub fn new(runtime_template: &ModuleCodeTemplate) -> Self {
    let require_name =
      runtime_template.render_runtime_globals_without_adding(&RuntimeGlobals::REQUIRE);
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
      require_regex: &LEGACY_REQUIRE_REGEX,
      context: context_name,
      module: module_name,
      rspack_module: rspack_module_name,
      exports: exports_name,
      require: require_name,
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
#[derive(Default)]
pub struct JavaScriptParserAndGenerator {
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
      && let Ok(collected_ts_info) =
        (collected_ts_info as Box<dyn std::any::Any>).downcast::<CollectedTypeScriptInfo>()
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
    let mut comments = Comments::default();
    let parser_lexer = Lexer::new(
      &allocator,
      Syntax::Es(EsSyntax {
        jsx,
        allow_return_outside_function: matches!(
          module_type,
          ModuleType::JsDynamic | ModuleType::JsAuto
        ),
        explicit_resource_management: true,
        import_attributes: true,
        ..Default::default()
      }),
      EsVersion::EsNext,
      StringSource::new(source_string.as_ref()),
      // The parser keeps this mutable borrow for the AST lifetime. We only read
      // the comments after dropping the parser below.
      Some(&mut comments),
    );
    let parser_lexer = Capturing::new(parser_lexer);
    let mut parser = Parser::new_from(&allocator, parser_lexer);

    let mut program = match match module_type {
      ModuleType::JsEsm => parser
        .parse_module()
        .map(|module| Program::Module(allocator.boxed(module))),
      ModuleType::JsDynamic => parser
        .parse_commonjs()
        .map(|script| Program::Script(allocator.boxed(script))),
      _ => parser.parse_program(),
    } {
      Ok(program) => program,
      Err(e) => {
        let mut errors = parser.take_errors();
        errors.push(e);
        append_experimental_parse_errors(&mut diagnostics, &source_string, errors);
        return default_with_diagnostics(source, diagnostics);
      }
    };

    let parse_errors = parser.take_errors();
    let tokens = parser.input_mut().iter.take();
    drop(parser);
    if !parse_errors.is_empty() {
      append_experimental_parse_errors(&mut diagnostics, &source_string, parse_errors);
      return default_with_diagnostics(source, diagnostics);
    }

    let mut semicolons = Default::default();
    remove_paren(&mut program, &allocator, Some(&mut comments));
    let semantic = resolver(&program);
    program.visit_with(&mut semicolon::InsertedSemicolons::new(
      &mut semicolons,
      &tokens,
    ));
    let parsed_ast = ParsedJavaScriptAst {
      allocator: &allocator,
      comments: &comments,
      semantic: &semantic,
      program: &program,
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
      build_meta.side_effect_free = Some(!has_side_effects);
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

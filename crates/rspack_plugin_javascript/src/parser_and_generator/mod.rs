use std::{
  borrow::Cow,
  collections::HashSet,
  sync::{Arc, LazyLock},
};

use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsyncDependenciesBlockIdentifier, BuildMetaExportsType, COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY,
  CachedConstDependency, ChunkGraph, CodeGenerationData, CollectedTypeScriptInfo, Compilation,
  ConstDependency, ContextDependency, ContextMode, DEFAULT_EXPORT, DependenciesBlock, Dependency,
  DependencyCodeGeneration, DependencyId, DependencyRange, ExportMode, ExportsArgument,
  ExportsType, GenerateContext, ImportPhase, JavascriptParserUrl, Module, ModuleArgument,
  ModuleCodeTemplate, ModuleGraph, ModuleType, ParseContext, ParseResult, ParserAndGenerator,
  RuntimeCondition, RuntimeGlobals, RuntimeRequirementsDependency,
  RuntimeRequirementsDependencyMode, RuntimeVariable, SideEffectsBailoutItem, SourceType,
  TemplateContext, TemplateReplaceSource, UsedName,
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics,
  property_access, property_access_with_optional, remove_bom, render_init_fragments,
  rspack_sources::{
    BoxSource, MapOptions, ObjectPool, RawStringSource, ReplaceSource, Source, SourceExt,
  },
  to_normal_comment,
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
  dependency::{
    AMDRequireContextDependency, CommonJsExportRequireDependency, CommonJsExportsDependency,
    CommonJsFullRequireDependency, CommonJsRequireContextDependency, CommonJsRequireDependency,
    CommonJsSelfReferenceDependency, CreateScriptUrlDependency, DeclarationId, ESMAcceptDependency,
    ESMCompatibilityDependency, ESMExportExpressionDependency, ESMExportHeaderDependency,
    ESMExportImportedSpecifierDependency, ESMExportSpecifierDependency,
    ESMImportSideEffectDependency, ESMImportSpecifierDependency,
    ESMImportSpecifierDependencyTemplate, ExportInfoDependency, ExternalModuleDependency,
    IMPORT_META_RSC_BINDING, ImportContextDependency, ImportDependency, ImportEagerDependency,
    ImportMetaContextDependency, ImportMetaHotAcceptDependency, ImportMetaHotDeclineDependency,
    ImportMetaResolveContextDependency, ImportMetaResolveDependency,
    ImportMetaResolveHeaderDependency, ImportMetaRscDependency, ImportWeakDependency,
    IsIncludeDependency, ModuleArgumentDependency, ModuleDecoratorDependency,
    ModuleHotAcceptDependency, ModuleHotDeclineDependency, ProvideDependency,
    PureExpressionDependency, RequireContextDependency, RequireEnsureDependency,
    RequireHeaderDependency, RequireMainDependency, RequireResolveContextDependency,
    RequireResolveDependency, RequireResolveHeaderDependency, URLContextDependency, URLDependency,
    WorkerDependency, amd_define_dependency::AMDDefineDependency,
    amd_require_array_dependency::AMDRequireArrayDependency,
    amd_require_dependency::AMDRequireDependency,
    amd_require_item_dependency::AMDRequireItemDependency, esm_import_dependency_apply,
    esm_import_dependency_prime_import_var, import_emitted_runtime,
    local_module_dependency::LocalModuleDependency, unsupported_dependency::UnsupportedDependency,
  },
  is_export_inlined,
  parser_plugin::JS_DEFAULT_KEYWORD,
  visitors::{ParsedJavaScriptAst, ScanDependenciesResult, scan_dependencies, semicolon},
};

mod ast_dependency;

use ast_dependency::{
  AstDependencyAction, AstDependencyRenderEntry, AstDependencyRenderKey, AstDependencyRenderPlan,
  AstDependencySideEffect, apply_ast_dependency_replacements, mark_ast_dependencies,
  render_ast_dependencies,
};

#[derive(Debug)]
pub struct ParserRuntimeRequirementsData {
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
    let rspack_module_name = runtime_template.render_runtime_variable(&RuntimeVariable::Module);
    Self {
      require_regex: &LEGACY_REQUIRE_REGEX,
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
    applied_ast_dependency_ids: &HashSet<DependencyId>,
  ) {
    let module_graph = compilation.get_module_graph();
    let block = module_graph
      .block_by_id(block_id)
      .expect("should have block");
    //    let block = block_id.expect_get(compilation);
    block.get_dependencies().iter().for_each(|dependency_id| {
      if applied_ast_dependency_ids.contains(dependency_id) {
        self.source_ast_dependency(compilation, dependency_id, source, context)
      } else {
        self.source_dependency(compilation, dependency_id, source, context)
      }
    });
    block.get_blocks().iter().for_each(|block_id| {
      self.source_block(
        compilation,
        block_id,
        source,
        context,
        applied_ast_dependency_ids,
      )
    });
  }

  fn collect_ast_dependency<'a>(
    &self,
    compilation: &'a Compilation,
    dependency_id: &DependencyId,
    entries: &mut Vec<AstDependencyRenderEntry>,
  ) {
    if let Some(dependency) = compilation
      .get_module_graph()
      .dependency_by_id(dependency_id)
      .as_dependency_code_generation()
      && let Some(entry) = AstDependencyRenderEntry::new(
        AstDependencyRenderKey::Dependency(*dependency_id),
        dependency,
      )
    {
      entries.push(entry);
    }
  }

  fn collect_ast_block<'a>(
    &self,
    compilation: &'a Compilation,
    block_id: &AsyncDependenciesBlockIdentifier,
    entries: &mut Vec<AstDependencyRenderEntry>,
  ) {
    let module_graph = compilation.get_module_graph();
    let block = module_graph
      .block_by_id(block_id)
      .expect("should have block");
    block
      .get_dependencies()
      .iter()
      .for_each(|dependency_id| self.collect_ast_dependency(compilation, dependency_id, entries));
    block
      .get_blocks()
      .iter()
      .for_each(|block_id| self.collect_ast_block(compilation, block_id, entries));
  }

  fn collect_ast_render_dependency(
    &self,
    compilation: &Compilation,
    dependency_id: &DependencyId,
    context: &mut TemplateContext,
    plan: &mut AstDependencyRenderPlan,
  ) -> bool {
    let Some(dependency) = compilation
      .get_module_graph()
      .dependency_by_id(dependency_id)
      .as_dependency_code_generation()
    else {
      return true;
    };

    self.collect_ast_render_action(
      dependency,
      context,
      plan,
      Some(AstDependencyRenderKey::Dependency(*dependency_id)),
    )
  }

  fn collect_ast_render_block(
    &self,
    compilation: &Compilation,
    block_id: &AsyncDependenciesBlockIdentifier,
    context: &mut TemplateContext,
    plan: &mut AstDependencyRenderPlan,
  ) -> bool {
    let module_graph = compilation.get_module_graph();
    let block = module_graph
      .block_by_id(block_id)
      .expect("should have block");
    block.get_dependencies().iter().all(|dependency_id| {
      self.collect_ast_render_dependency(compilation, dependency_id, context, plan)
    }) && block
      .get_blocks()
      .iter()
      .all(|block_id| self.collect_ast_render_block(compilation, block_id, context, plan))
  }

  fn collect_ast_render_plan(
    &self,
    module: &dyn Module,
    context: &mut TemplateContext,
  ) -> Option<AstDependencyRenderPlan> {
    let mut plan = AstDependencyRenderPlan::default();
    let compilation = context.compilation;

    let supported = module.get_dependencies().iter().all(|dependency_id| {
      self.collect_ast_render_dependency(compilation, dependency_id, context, &mut plan)
    }) && module
      .get_presentational_dependencies()
      .map(|dependencies| {
        dependencies.iter().enumerate().all(|(idx, dependency)| {
          self.collect_ast_render_action(
            dependency.as_ref(),
            context,
            &mut plan,
            Some(AstDependencyRenderKey::Presentational(idx)),
          )
        })
      })
      .unwrap_or(true)
      && module
        .get_blocks()
        .iter()
        .all(|block_id| self.collect_ast_render_block(compilation, block_id, context, &mut plan));

    supported.then_some(plan)
  }

  fn render_ast_amd_require_array(
    &self,
    context: &TemplateContext,
    dep: &AMDRequireArrayDependency,
  ) -> String {
    let mut init_fragments = Vec::new();
    let mut data = CodeGenerationData::default();
    let mut runtime_template = context.runtime_template.clone();
    let mut temp_context = TemplateContext {
      compilation: context.compilation,
      module: context.module,
      init_fragments: &mut init_fragments,
      runtime: context.runtime,
      concatenation_scope: None,
      data: &mut data,
      runtime_template: &mut runtime_template,
    };

    dep.content(&mut temp_context)
  }

  fn render_ast_context_module_raw(
    &self,
    context: &TemplateContext,
    dep: &dyn ContextDependency,
    weak: bool,
  ) -> String {
    let mut runtime_template = context.runtime_template.clone();
    runtime_template.module_raw(context.compilation, dep.id(), dep.request(), weak)
  }

  fn render_ast_template_replacements(
    &self,
    context: &mut TemplateContext,
    dependency: &dyn DependencyCodeGeneration,
    validate_range: DependencyRange,
  ) -> Option<Vec<(String, u32, u32)>> {
    let template = dependency
      .dependency_template()
      .and_then(|template_type| context.compilation.get_dependency_template(template_type))?;
    let mut init_fragments = Vec::new();
    let mut data = CodeGenerationData::default();
    let mut runtime_template = context.runtime_template.clone();
    let mut temp_context = TemplateContext {
      compilation: context.compilation,
      module: context.module,
      init_fragments: &mut init_fragments,
      runtime: context.runtime,
      concatenation_scope: context
        .concatenation_scope
        .as_mut()
        .map(|scope| &mut **scope),
      data: &mut data,
      runtime_template: &mut runtime_template,
    };
    let mut source =
      ReplaceSource::new(RawStringSource::from(" ".repeat(validate_range.end as usize)).boxed());
    template.render(dependency, &mut source, &mut temp_context);
    Some(
      source
        .replacements()
        .iter()
        .map(|replacement| {
          (
            replacement.content().to_string(),
            replacement.start(),
            replacement.end(),
          )
        })
        .collect(),
    )
  }

  fn push_template_replacement_actions(
    &self,
    plan: &mut AstDependencyRenderPlan,
    validate_range: DependencyRange,
    replacements: Vec<(String, u32, u32)>,
  ) -> bool {
    if replacements.is_empty() {
      return true;
    }

    let action = if replacements.len() == 1
      && replacements[0].1 == validate_range.start
      && replacements[0].2 == validate_range.end
    {
      let (content, _, _) = replacements.into_iter().next().expect("checked len");
      AstDependencyAction::expr(validate_range, content)
    } else {
      AstDependencyAction::validated_replacements(validate_range, replacements)
    };
    let Some(action) = action else {
      return false;
    };
    plan.push_action(action);
    true
  }

  fn collect_ast_context_require_call_action(
    &self,
    context: &TemplateContext,
    plan: &mut AstDependencyRenderPlan,
    key: Option<AstDependencyRenderKey>,
    dep: &dyn ContextDependency,
    range: DependencyRange,
    value_range: Option<DependencyRange>,
  ) -> bool {
    let expr = self.render_ast_context_module_raw(context, dep, false);
    let module_exists = context
      .compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(dep.id())
      .is_some();

    let action = if module_exists {
      if let Some(value_range) = value_range {
        AstDependencyAction::wrapped_source_with_replacements(
          range,
          value_range,
          format!("{expr}("),
          ")",
          dep.options().replaces.clone(),
        )
      } else {
        AstDependencyAction::expr(range, expr)
      }
    } else {
      AstDependencyAction::expr(range, expr)
    };

    let Some(action) = action else {
      return false;
    };
    let Some(key) = key else {
      return false;
    };
    plan.push_action(action);
    plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
    true
  }

  fn collect_ast_context_id_action(
    &self,
    context: &TemplateContext,
    plan: &mut AstDependencyRenderPlan,
    key: Option<AstDependencyRenderKey>,
    dep: &dyn ContextDependency,
    range: DependencyRange,
  ) -> bool {
    let expr =
      self.render_ast_context_module_raw(context, dep, dep.options().mode == ContextMode::Weak);
    let module_exists = context
      .compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(dep.id())
      .is_some();

    let action = if module_exists {
      AstDependencyAction::wrapped_source_with_replacements(
        range,
        range,
        format!("{expr}.resolve("),
        ")",
        dep.options().replaces.clone(),
      )
    } else {
      AstDependencyAction::expr(range, expr)
    };

    let Some(action) = action else {
      return false;
    };
    let Some(key) = key else {
      return false;
    };
    plan.push_action(action);
    plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
    true
  }

  fn render_ast_esm_import_specifier(
    &self,
    context: &TemplateContext,
    dep: &ESMImportSpecifierDependency,
  ) -> Option<Option<String>> {
    let module_graph = context.compilation.get_module_graph();
    let ids = dep.get_ids(module_graph);
    let connection = module_graph.connection_by_dependency_id(dep.id());
    if let Some(con) = connection
      && !con.is_target_active(
        module_graph,
        context.runtime,
        &context.compilation.module_graph_cache_artifact,
        &context
          .compilation
          .build_module_graph_artifact
          .side_effects_state_artifact,
        &context.compilation.exports_info_artifact,
      )
      && !is_export_inlined(
        &context.compilation.exports_info_artifact,
        con.module_identifier(),
        ids,
        context.runtime,
      )
    {
      return Some(None);
    }

    let mut init_fragments = Vec::new();
    let mut data = CodeGenerationData::default();
    let mut runtime_template = context.runtime_template.clone();
    let mut concatenation_scope = context
      .concatenation_scope
      .as_ref()
      .map(|scope| (**scope).clone());
    let mut temp_context = TemplateContext {
      compilation: context.compilation,
      module: context.module,
      init_fragments: &mut init_fragments,
      runtime: context.runtime,
      concatenation_scope: concatenation_scope.as_mut(),
      data: &mut data,
      runtime_template: &mut runtime_template,
    };
    let template = ESMImportSpecifierDependencyTemplate::default();
    if dep.evaluated_in_operator {
      template
        .get_evaluated_in_operator_code(ids, dep, connection, &mut temp_context)
        .map(Some)
    } else {
      Some(Some(template.get_code_for_ids(
        ids,
        dep,
        connection,
        &mut temp_context,
      )))
    }
  }

  fn collect_ast_esm_import_specifier_destructuring_actions(
    &self,
    context: &TemplateContext,
    dep: &ESMImportSpecifierDependency,
    plan: &mut AstDependencyRenderPlan,
  ) -> bool {
    let Some(referenced_properties) = dep.referenced_properties_in_destructuring() else {
      return true;
    };

    let module_graph = context.compilation.get_module_graph();
    let ids = dep.get_ids(module_graph);
    let mut prefixed_ids = ids.to_vec();

    let Some(module) = module_graph.get_module_by_dependency_id(dep.id()) else {
      return true;
    };

    if ids.first().is_some_and(|id| id == "default") {
      let Some(self_module) = module_graph
        .get_parent_module(dep.id())
        .and_then(|id| module_graph.module_by_identifier(id))
      else {
        return false;
      };
      let exports_type = module.get_exports_type(
        module_graph,
        &context.compilation.module_graph_cache_artifact,
        &context.compilation.exports_info_artifact,
        self_module.build_meta().strict_esm_module,
      );
      if matches!(
        exports_type,
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
      ) && !ids.is_empty()
      {
        prefixed_ids = ids[1..].to_vec();
      }
    }

    let mut actions = Vec::new();
    referenced_properties.traverse_on_enter(&mut |stack| {
      let prop = stack.last().expect("should have last");
      let mut concated_ids = prefixed_ids.clone();
      concated_ids.extend(stack.iter().map(|p| p.id.clone()));
      let Some(new_name) = context
        .compilation
        .exports_info_artifact
        .get_exports_info_data(&module.identifier())
        .get_used_name(
          &context.compilation.exports_info_artifact,
          context.runtime,
          &concated_ids,
        )
        .and_then(|used| match used {
          UsedName::Normal(names) => names.last().cloned(),
          UsedName::Inlined(inlined) => {
            unreachable!("should not inline for destructuring {:#?}", inlined)
          }
        })
      else {
        return;
      };

      if new_name == prop.id {
        return;
      }

      let comment = to_normal_comment(prop.id.as_str());
      let key = format!("{comment}{new_name}");
      let content = if prop.shorthand {
        format!("{key}: {}", prop.id)
      } else {
        key
      };
      actions.push((prop.range, content));
    });

    for (range, content) in actions {
      let Some(action) = AstDependencyAction::raw_ident(range, content) else {
        return false;
      };
      plan.push_action(action);
    }

    true
  }

  fn prime_ast_esm_import_side_effect(
    &self,
    context: &TemplateContext,
    dep: &ESMImportSideEffectDependency,
  ) {
    let module_graph = context.compilation.get_module_graph();
    let module = module_graph.get_module_by_dependency_id(dep.id());

    if module.is_none() && !dep.missing_module_active() {
      return;
    }

    if let Some(module) = module {
      let source_types = module.source_types(module_graph);
      if source_types
        .iter()
        .all(|source_type| matches!(source_type, SourceType::Css))
      {
        return;
      }
    }

    if let Some(scope) = context.concatenation_scope.as_ref()
      && module.is_some_and(|m| scope.is_module_in_scope(&m.identifier()))
    {
      return;
    }

    let mut init_fragments = Vec::new();
    let mut data = CodeGenerationData::default();
    let mut runtime_template = context.runtime_template.clone();
    let mut temp_context = TemplateContext {
      compilation: context.compilation,
      module: context.module,
      init_fragments: &mut init_fragments,
      runtime: context.runtime,
      concatenation_scope: None,
      data: &mut data,
      runtime_template: &mut runtime_template,
    };
    esm_import_dependency_apply(dep, dep.source_order(), dep.phase(), &mut temp_context);
  }

  fn prime_ast_esm_export_imported_specifier(
    &self,
    context: &TemplateContext,
    dep: &ESMExportImportedSpecifierDependency,
  ) {
    if context.concatenation_scope.is_some() {
      return;
    }

    let module_graph = context.compilation.get_module_graph();
    let mode = dep.get_mode(
      module_graph,
      context.runtime,
      &context.compilation.module_graph_cache_artifact,
      &context.compilation.exports_info_artifact,
    );

    if matches!(
      mode,
      ExportMode::LazyMake | ExportMode::Unused(_) | ExportMode::EmptyStar(_)
    ) {
      return;
    }

    let _ = esm_import_dependency_prime_import_var(dep, dep.phase(), context);
  }

  fn render_ast_esm_export_expression_head(&self, context: &TemplateContext) -> String {
    let supports_const = context
      .compilation
      .options
      .output
      .environment
      .supports_const();

    if context.concatenation_scope.is_some() {
      return format!(
        "/* export default */ {} {DEFAULT_EXPORT} = ",
        if supports_const { "const" } else { "var" }
      );
    }

    if let Some(used) = context
      .compilation
      .exports_info_artifact
      .get_exports_info_data(&context.module.identifier())
      .get_used_name(
        &context.compilation.exports_info_artifact,
        context.runtime,
        std::slice::from_ref(&JS_DEFAULT_KEYWORD),
      )
    {
      if let UsedName::Normal(used) = used {
        if supports_const {
          format!("/* export default */ const {DEFAULT_EXPORT} = ")
        } else {
          let exports_argument = match context.module.get_exports_argument() {
            ExportsArgument::Exports => "exports".to_string(),
            ExportsArgument::RspackExports => context
              .runtime_template
              .render_runtime_variable(&RuntimeVariable::Exports),
          };
          format!(
            "/* export default */ {}{} = ",
            exports_argument,
            property_access(used, 0)
          )
        }
      } else {
        format!("/* inlined export default */ var {DEFAULT_EXPORT} = ")
      }
    } else {
      format!("/* unused export default */ var {DEFAULT_EXPORT} = ")
    }
  }

  fn collect_ast_esm_export_expression_action(
    &self,
    context: &TemplateContext,
    plan: &mut AstDependencyRenderPlan,
    key: Option<AstDependencyRenderKey>,
    dep: &ESMExportExpressionDependency,
  ) -> bool {
    let range = dep.range();
    let range_stmt = dep.range_stmt();
    let mut replacements = Vec::new();

    if let Some(declaration) = dep.declaration() {
      replacements.push((
        format!("/* export default */ {}", dep.prefix()),
        range_stmt.start,
        range.start,
      ));

      if let DeclarationId::Func(func) = declaration {
        let func_range = func.range();
        replacements.push((
          format!("{}{}{}", func.prefix(), DEFAULT_EXPORT, func.suffix()),
          func_range.start,
          func_range.end,
        ));
      }
    } else {
      replacements.push((
        format!(
          "{}({}",
          self.render_ast_esm_export_expression_head(context),
          dep.prefix()
        ),
        range_stmt.start,
        range.start,
      ));
      replacements.push((");".to_string(), range.end, range_stmt.end));
    }

    let Some(action) =
      AstDependencyAction::source_with_replacements(range_stmt, range_stmt, replacements)
    else {
      return false;
    };
    let Some(key) = key else {
      return false;
    };
    plan.push_action(action);
    plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
    true
  }

  fn collect_ast_require_ensure_action(
    &self,
    context: &TemplateContext,
    plan: &mut AstDependencyRenderPlan,
    key: Option<AstDependencyRenderKey>,
    dep: &RequireEnsureDependency,
  ) -> bool {
    let module_graph = context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(dep.id());
    let mut runtime_template = context.runtime_template.clone();
    let promise =
      runtime_template.block_promise(block, context.compilation, dep.dependency_type().as_str());
    let require = context
      .runtime_template
      .render_runtime_globals_without_adding(&RuntimeGlobals::REQUIRE);
    let range = dep.range();
    let content_range = dep.content_range();
    let mut replacements = vec![(
      format!("{promise}.then(("),
      range.start,
      content_range.start,
    )];

    if let Some(error_handler_range) = dep.error_handler_range() {
      replacements.push((
        format!(").bind(null, {require}))['catch']("),
        content_range.end,
        error_handler_range.start,
      ));
      replacements.push((")".to_string(), error_handler_range.end, range.end));
    } else {
      replacements.push((
        format!(
          ").bind(null, {require}))['catch']({})",
          context
            .runtime_template
            .render_runtime_globals_without_adding(&RuntimeGlobals::UNCAUGHT_ERROR_HANDLER)
        ),
        content_range.end,
        range.end,
      ));
    }

    let Some(action) = AstDependencyAction::source_with_replacements(range, range, replacements)
    else {
      return false;
    };
    let Some(key) = key else {
      return false;
    };
    plan.push_action(action);
    plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
    true
  }

  fn collect_ast_amd_require_action(
    &self,
    context: &TemplateContext,
    plan: &mut AstDependencyRenderPlan,
    key: Option<AstDependencyRenderKey>,
    dep: &AMDRequireDependency,
  ) -> bool {
    let module_graph = context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(dep.id());
    let mut runtime_template = context.runtime_template.clone();
    let promise = runtime_template.block_promise(block, context.compilation, "AMD require");
    let uncaught_error_handler = context
      .runtime_template
      .render_runtime_globals_without_adding(&RuntimeGlobals::UNCAUGHT_ERROR_HANDLER);
    let require = context
      .runtime_template
      .render_runtime_globals_without_adding(&RuntimeGlobals::REQUIRE);
    let outer_range = dep.outer_range();
    let mut replacements = Vec::new();

    match (
      dep.array_range(),
      dep.function_range(),
      dep.error_callback_range(),
    ) {
      (Some(array_range), None, _) => {
        replacements.push((
          format!("{promise}.then(function() {{"),
          outer_range.start,
          array_range.start,
        ));
        replacements.push((
          format!(";}})['catch']({uncaught_error_handler})"),
          array_range.end,
          outer_range.end,
        ));
      }
      (None, Some(function_range), _) => {
        replacements.push((
          format!("{promise}.then(("),
          outer_range.start,
          function_range.start,
        ));
        replacements.push((
          format!(
            ").bind(exports, {require}, exports, module))['catch']({uncaught_error_handler})"
          ),
          function_range.end,
          outer_range.end,
        ));
      }
      (Some(array_range), Some(function_range), Some(error_callback_range)) => {
        replacements.push((
          format!("{promise}.then(function() {{ "),
          outer_range.start,
          array_range.start,
        ));
        replacements.push((
          "var __rspack_amd_require_deps = ".to_string(),
          array_range.start,
          array_range.start,
        ));
        replacements.push(("; (".to_string(), array_range.end, function_range.start));
        replacements.push((
          ").apply(null, __rspack_amd_require_deps);".to_string(),
          function_range.end,
          function_range.end,
        ));
        replacements.push((
          if dep.function_bind_this {
            "}.bind(this))['catch'](".to_string()
          } else {
            "})['catch'](".to_string()
          },
          function_range.end,
          error_callback_range.start,
        ));
        replacements.push((
          if dep.error_callback_bind_this {
            ".bind(this))".to_string()
          } else {
            ")".to_string()
          },
          error_callback_range.end,
          outer_range.end,
        ));
      }
      (Some(array_range), Some(function_range), None) => {
        replacements.push((
          format!("{promise}.then(function() {{ "),
          outer_range.start,
          array_range.start,
        ));
        replacements.push((
          "var __rspack_amd_require_deps = ".to_string(),
          array_range.start,
          array_range.start,
        ));
        replacements.push(("; (".to_string(), array_range.end, function_range.start));
        replacements.push((
          ").apply(null, __rspack_amd_require_deps);".to_string(),
          function_range.end,
          function_range.end,
        ));
        replacements.push((
          format!(
            "}}{})['catch']({uncaught_error_handler})",
            if dep.function_bind_this {
              ".bind(this)"
            } else {
              ""
            }
          ),
          function_range.end,
          outer_range.end,
        ));
      }
      (None, None, _) => return false,
    }

    let Some(action) =
      AstDependencyAction::source_with_replacements(outer_range, outer_range, replacements)
    else {
      return false;
    };
    let Some(key) = key else {
      return false;
    };
    plan.push_action(action);
    plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
    true
  }

  fn render_ast_esm_accept_content(
    &self,
    context: &TemplateContext,
    dep: &ESMAcceptDependency,
  ) -> String {
    let mut content = String::new();
    let module_graph = context.compilation.get_module_graph();
    let module_identifier = context.module.identifier();
    let mut runtime_template = context.runtime_template.clone();

    for id in dep.dependency_ids() {
      let dependency = module_graph.dependency_by_id(id);
      let target_module = module_graph.get_module_by_dependency_id(dependency.id());
      let runtime_condition = match target_module {
        Some(target_module) => {
          import_emitted_runtime::get_runtime(&module_identifier, &target_module.identifier())
        }
        None => RuntimeCondition::Boolean(false),
      };

      if matches!(runtime_condition, RuntimeCondition::Boolean(false)) {
        continue;
      }

      let condition = runtime_template.runtime_condition_expression(
        &context.compilation.build_chunk_graph_artifact.chunk_graph,
        Some(&runtime_condition),
        context.runtime,
      );

      let module_dependency = dependency
        .as_module_dependency()
        .expect("should be module dependency");
      let phase = ImportPhase::Evaluation;
      let import_var = context.compilation.get_import_var(
        module_identifier,
        target_module,
        module_dependency.user_request(),
        phase,
        context.runtime,
      );
      let stmts = runtime_template.import_statement(
        context.module,
        context.compilation,
        id,
        &import_var,
        module_dependency.request(),
        phase,
        true,
      );
      if condition == "true" {
        content.push_str(&stmts.0);
        content.push_str(&stmts.1);
      } else {
        content.push_str(&format!("if ({condition}) {{\n"));
        content.push_str(&stmts.0);
        content.push_str(&stmts.1);
        content.push_str("\n}\n");
      }
    }

    content
  }

  fn collect_ast_esm_accept_action(
    &self,
    context: &TemplateContext,
    plan: &mut AstDependencyRenderPlan,
    key: Option<AstDependencyRenderKey>,
    dep: &ESMAcceptDependency,
  ) -> bool {
    let content = self.render_ast_esm_accept_content(context, dep);
    let action = if dep.has_callback() {
      let range = dep.range();
      AstDependencyAction::wrapped_source(
        range,
        range,
        format!("function(__rspack_hmr_outdated) {{\n{content}("),
        ")(__rspack_hmr_outdated); }.bind(this)",
      )
    } else {
      AstDependencyAction::insert(
        dep.call_range(),
        dep.range().start,
        format!(", function(){{\n{content}\n}}"),
      )
    };
    let Some(action) = action else {
      return false;
    };
    let Some(key) = key else {
      return false;
    };
    plan.push_action(action);
    plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
    true
  }

  fn render_ast_worker_import(
    &self,
    context: &TemplateContext,
    dep: &WorkerDependency,
  ) -> Option<String> {
    let chunk_id = context
      .compilation
      .get_module_graph()
      .get_parent_block(dep.id())
      .and_then(|block| {
        context
          .compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_block_chunk_group(
            block,
            &context
              .compilation
              .build_chunk_graph_artifact
              .chunk_group_by_ukey,
          )
      })
      .map(|entrypoint| entrypoint.get_entrypoint_chunk())
      .and_then(|ukey| {
        context
          .compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .get(&ukey)
      })
      .and_then(|chunk| chunk.id())
      .map(rspack_util::json_stringify)?;
    let worker_import_base_url = if !dep.public_path().is_empty() {
      format!("\"{}\"", dep.public_path())
    } else {
      context
        .runtime_template
        .render_runtime_globals_without_adding(&RuntimeGlobals::PUBLIC_PATH)
    };

    let worker_import_str = format!(
      "/* worker import */{} + {}({}), {}",
      worker_import_base_url,
      context
        .runtime_template
        .render_runtime_globals_without_adding(&RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
      chunk_id,
      context
        .runtime_template
        .render_runtime_globals_without_adding(&RuntimeGlobals::BASE_URI)
    );

    if dep.need_new_url() {
      Some(format!("new URL({worker_import_str})"))
    } else {
      Some(worker_import_str)
    }
  }

  fn collect_ast_render_action(
    &self,
    dependency: &dyn DependencyCodeGeneration,
    context: &mut TemplateContext,
    plan: &mut AstDependencyRenderPlan,
    key: Option<AstDependencyRenderKey>,
  ) -> bool {
    if dependency.dependency_template().is_none() {
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ESMImportSideEffectDependency>()
    {
      self.prime_ast_esm_import_side_effect(context, dep);
      let Some(key) = key else {
        return false;
      };
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if dependency
      .as_any()
      .downcast_ref::<ESMCompatibilityDependency>()
      .is_some()
      || dependency
        .as_any()
        .downcast_ref::<ESMExportSpecifierDependency>()
        .is_some()
    {
      let Some(key) = key else {
        return false;
      };
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ESMExportImportedSpecifierDependency>()
    {
      self.prime_ast_esm_export_imported_specifier(context, dep);
      let Some(key) = key else {
        return false;
      };
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if dependency
      .as_any()
      .downcast_ref::<ExternalModuleDependency>()
      .is_some()
      || dependency
        .as_any()
        .downcast_ref::<ModuleDecoratorDependency>()
        .is_some()
    {
      let Some(key) = key else {
        return false;
      };
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ImportMetaRscDependency>()
    {
      if let Some(range) = dep.ast_dependency_range() {
        let Some(action) = AstDependencyAction::expr(range, IMPORT_META_RSC_BINDING) else {
          return false;
        };
        plan.push_action(action);
      }
      let Some(key) = key else {
        return false;
      };
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<CommonJsRequireContextDependency>()
    {
      return self.collect_ast_context_require_call_action(
        context,
        plan,
        key,
        dep,
        dep.range(),
        dep.value_range(),
      );
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ImportContextDependency>()
    {
      return self.collect_ast_context_require_call_action(
        context,
        plan,
        key,
        dep,
        dep.range(),
        Some(dep.value_range()),
      );
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<URLContextDependency>() {
      return self.collect_ast_context_require_call_action(
        context,
        plan,
        key,
        dep,
        dep.range(),
        Some(dep.value_range()),
      );
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<AMDRequireContextDependency>()
    {
      return self.collect_ast_context_require_call_action(
        context,
        plan,
        key,
        dep,
        dep.range(),
        Some(dep.range()),
      );
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<RequireResolveContextDependency>()
    {
      return self.collect_ast_context_id_action(context, plan, key, dep, dep.range());
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ImportMetaResolveContextDependency>()
    {
      return self.collect_ast_context_id_action(context, plan, key, dep, dep.range());
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<URLDependency>()
      && !matches!(
        dep.mode(),
        Some(JavascriptParserUrl::Relative | JavascriptParserUrl::NewUrlRelative)
      )
    {
      let Some(replacements) =
        self.render_ast_template_replacements(context, dependency, dep.range())
      else {
        return false;
      };
      if !self.push_template_replacement_actions(plan, dep.range(), replacements) {
        return false;
      }
      let Some(key) = key else {
        return false;
      };
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<AMDDefineDependency>() {
      let mut runtime_template = context.runtime_template.clone();
      let Some((definition, replacements)) = dep.ast_define_replacements(&mut runtime_template)
      else {
        return false;
      };
      if let Some(definition) = definition {
        let Some(action) = AstDependencyAction::insert(dep.range(), 0, definition) else {
          return false;
        };
        plan.push_action(action);
      }
      let Some(action) = AstDependencyAction::validated_replacements(dep.range(), replacements)
      else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ESMImportSpecifierDependency>()
    {
      let Some(content) = self.render_ast_esm_import_specifier(context, dep) else {
        return false;
      };
      let Some(content) = content else {
        return true;
      };
      let Some(range) = dep.range() else {
        return false;
      };
      let action = if dep.shorthand() {
        AstDependencyAction::raw_ident_with_suffix(range, format!(": {content}"))
      } else {
        AstDependencyAction::expr(range, content.into_boxed_str())
      };
      let Some(action) = action else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      if !self.collect_ast_esm_import_specifier_destructuring_actions(context, dep, plan) {
        return false;
      }
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ESMExportExpressionDependency>()
    {
      return self.collect_ast_esm_export_expression_action(context, plan, key, dep);
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ESMExportHeaderDependency>()
    {
      let range = dep.range();
      let action = if let Some(range_decl) = dep.range_decl() {
        AstDependencyAction::validated_replacements(
          range,
          vec![("".to_string(), range.start, range_decl.start)],
        )
      } else {
        AstDependencyAction::expr(range, "")
      };
      let Some(action) = action else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<RuntimeRequirementsDependency>()
      && matches!(dep.mode, RuntimeRequirementsDependencyMode::AddOnly)
    {
      plan.push_side_effect(AstDependencySideEffect::RuntimeRequirements(
        dep.runtime_requirements,
      ));
      return true;
    }

    let Some(range) = dependency.ast_dependency_range() else {
      return false;
    };

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<RequireEnsureDependency>()
    {
      return self.collect_ast_require_ensure_action(context, plan, key, dep);
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<ESMAcceptDependency>() {
      return self.collect_ast_esm_accept_action(context, plan, key, dep);
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<ConstDependency>() {
      let Some(action) = AstDependencyAction::expr(range, dep.content.clone()) else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<CachedConstDependency>() {
      let Some(action) = AstDependencyAction::expr(range, dep.identifier.clone()) else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::CachedConst {
        identifier: dep.identifier.clone(),
        content: dep.content.clone(),
      });
      return true;
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<LocalModuleDependency>() {
      let Some(action) = AstDependencyAction::expr(range, dep.module_instance().into_boxed_str())
      else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<AMDRequireItemDependency>()
    {
      let mut runtime_template = context.runtime_template.clone();
      let content =
        runtime_template.module_raw(context.compilation, dep.id(), dep.request(), false);
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<AMDRequireArrayDependency>()
    {
      let content = self.render_ast_amd_require_array(context, dep);
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<AMDRequireDependency>() {
      return self.collect_ast_amd_require_action(context, plan, key, dep);
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<UnsupportedDependency>() {
      let Some(action) = AstDependencyAction::expr(range, dep.content().into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<RequireContextDependency>()
    {
      let mut runtime_template = context.runtime_template.clone();
      let content =
        runtime_template.module_raw(context.compilation, dep.id(), dep.request(), dep.optional());
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ImportMetaContextDependency>()
    {
      let mut runtime_template = context.runtime_template.clone();
      let content =
        runtime_template.module_raw(context.compilation, dep.id(), dep.request(), dep.optional());
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<ProvideDependency>() {
      let Some(action) =
        AstDependencyAction::expr(range, dep.identifier().to_string().into_boxed_str())
      else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<RuntimeRequirementsDependency>()
    {
      let mut content = context
        .runtime_template
        .render_runtime_globals_without_adding(&dep.runtime_requirements);
      if matches!(dep.mode, RuntimeRequirementsDependencyMode::Call) {
        content = format!("{content}()");
      }
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RuntimeRequirements(
        dep.runtime_requirements,
      ));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<PureExpressionDependency>()
    {
      let (prefix, suffix, needs_side_effect) =
        match dep.get_runtime_condition(context.compilation, context.runtime) {
          RuntimeCondition::Boolean(true) => return true,
          RuntimeCondition::Boolean(false) => (
            "(/* unused pure expression or super */ null && (".to_string(),
            "))",
            false,
          ),
          RuntimeCondition::Spec(runtime_condition) => {
            let mut runtime_template = context.runtime_template.clone();
            let condition = runtime_template.runtime_condition_expression(
              &context.compilation.build_chunk_graph_artifact.chunk_graph,
              Some(&RuntimeCondition::Spec(runtime_condition)),
              context.runtime,
            );
            (
              format!("(/* runtime-dependent pure expression or super */ {condition} ? ("),
              ") : null)",
              true,
            )
          }
        };

      let Some(action) = AstDependencyAction::wrapped_source(range, range, prefix, suffix) else {
        return false;
      };
      plan.push_action(action);
      if needs_side_effect {
        let Some(key) = key else {
          return false;
        };
        plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      }
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ModuleHotAcceptDependency>()
    {
      let content = context.runtime_template.module_id(
        context.compilation,
        dep.id(),
        dep.request(),
        dep.weak(),
      );
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ModuleHotDeclineDependency>()
    {
      let content = context.runtime_template.module_id(
        context.compilation,
        dep.id(),
        dep.request(),
        dep.weak(),
      );
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ImportMetaHotAcceptDependency>()
    {
      let content = context.runtime_template.module_id(
        context.compilation,
        dep.id(),
        dep.request(),
        dep.weak(),
      );
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ImportMetaHotDeclineDependency>()
    {
      let content = context.runtime_template.module_id(
        context.compilation,
        dep.id(),
        dep.request(),
        dep.weak(),
      );
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if dependency
      .as_any()
      .downcast_ref::<RequireResolveHeaderDependency>()
      .is_some()
    {
      let Some(action) = AstDependencyAction::raw_expr(range, "/*require.resolve*/") else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if dependency
      .as_any()
      .downcast_ref::<ImportMetaResolveHeaderDependency>()
      .is_some()
    {
      let Some(action) = AstDependencyAction::raw_expr(range, "/*import.meta.resolve*/") else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<RequireResolveDependency>()
    {
      let content =
        context
          .runtime_template
          .module_id(context.compilation, &dep.id, &dep.request, dep.weak);
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ImportMetaResolveDependency>()
    {
      let content =
        context
          .runtime_template
          .module_id(context.compilation, &dep.id, &dep.request, false);
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<URLDependency>() {
      let Some(replacements) =
        self.render_ast_template_replacements(context, dependency, dep.range())
      else {
        return false;
      };
      if !self.push_template_replacement_actions(plan, dep.range(), replacements) {
        return false;
      }
      let Some(key) = key else {
        return false;
      };
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<CreateScriptUrlDependency>()
    {
      let Some(action) = AstDependencyAction::wrapped_source(
        range,
        dep.range_path(),
        format!(
          "{}(",
          context
            .runtime_template
            .render_runtime_globals_without_adding(&RuntimeGlobals::CREATE_SCRIPT_URL)
        ),
        ")",
      ) else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<WorkerDependency>() {
      let Some(content) = self.render_ast_worker_import(context, dep) else {
        return false;
      };
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if dependency
      .as_any()
      .downcast_ref::<RequireHeaderDependency>()
      .is_some()
    {
      let content = context
        .runtime_template
        .render_runtime_globals_without_adding(&RuntimeGlobals::REQUIRE);
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RuntimeRequirements(
        RuntimeGlobals::REQUIRE,
      ));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<CommonJsRequireDependency>()
    {
      let content =
        context
          .runtime_template
          .module_id(context.compilation, dep.id(), dep.request(), false);
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<CommonJsFullRequireDependency>()
    {
      let module_graph = context.compilation.get_module_graph();
      let require_expr = if let Some(imported_module) =
        module_graph.module_graph_module_by_dependency_id(dep.id())
        && let Some(used) = {
          let exports_info = context
            .compilation
            .exports_info_artifact
            .get_exports_info_data(&imported_module.module_identifier);
          exports_info.get_used_name(
            &context.compilation.exports_info_artifact,
            context.runtime,
            dep.names(),
          )
        } {
        let mut require_expr = match used {
          UsedName::Normal(used) => format!(
            "{}({}){}{}",
            context
              .runtime_template
              .render_runtime_globals_without_adding(&RuntimeGlobals::REQUIRE),
            context
              .runtime_template
              .module_id(context.compilation, dep.id(), dep.request(), false),
            to_normal_comment(&property_access(dep.names(), 0)),
            property_access(used, 0)
          ),
          UsedName::Inlined(inlined) => inlined.render(&to_normal_comment(&format!(
            "inlined export {}",
            property_access(dep.names(), 0)
          ))),
        };
        if dep.asi_safe() {
          require_expr = format!("({require_expr})");
        }
        require_expr
      } else {
        format!(
          "{}({})",
          context
            .runtime_template
            .render_runtime_globals_without_adding(&RuntimeGlobals::REQUIRE),
          context
            .runtime_template
            .module_id(context.compilation, dep.id(), dep.request(), false)
        )
      };
      let Some(action) = AstDependencyAction::expr(range, require_expr.into_boxed_str()) else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<CommonJsSelfReferenceDependency>()
    {
      let module_graph = context.compilation.get_module_graph();
      let module = module_graph
        .module_by_identifier(&context.module.identifier())
        .expect("should have mgm");

      let used = if !dep.names().is_empty() {
        let exports_info = context
          .compilation
          .exports_info_artifact
          .get_exports_info_data(&module.identifier());
        exports_info
          .get_used_name(
            &context.compilation.exports_info_artifact,
            context.runtime,
            dep.names(),
          )
          .unwrap_or_else(|| UsedName::Normal(dep.names().to_vec()))
      } else {
        UsedName::Normal(dep.names().to_vec())
      };

      let (base, runtime_requirements) = if dep.base().is_exports() {
        let base = match module.get_exports_argument() {
          ExportsArgument::Exports => "exports".to_string(),
          ExportsArgument::RspackExports => context
            .runtime_template
            .render_runtime_variable(&RuntimeVariable::Exports),
        };
        (base, RuntimeGlobals::EXPORTS)
      } else if dep.base().is_module_exports() {
        let module_argument = match module.get_module_argument() {
          ModuleArgument::Module => "module".to_string(),
          ModuleArgument::RspackModule => context
            .runtime_template
            .render_runtime_variable(&RuntimeVariable::Module),
        };
        (format!("{module_argument}.exports"), RuntimeGlobals::MODULE)
      } else if dep.base().is_this() {
        ("this".to_string(), RuntimeGlobals::THIS_AS_EXPORTS)
      } else {
        unreachable!();
      };

      let content = match used {
        UsedName::Normal(used) => {
          format!(
            "{}{}",
            base,
            property_access_with_optional(used, dep.names_optionals(), 0)
          )
        }
        UsedName::Inlined(inlined) => inlined.render(""),
      };
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RuntimeRequirements(
        runtime_requirements,
      ));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<CommonJsExportsDependency>()
    {
      let module_graph = context.compilation.get_module_graph();
      let module = module_graph
        .module_by_identifier(&context.module.identifier())
        .expect("should have mgm");

      let exports_info = context
        .compilation
        .exports_info_artifact
        .get_exports_info_data(&module.identifier());
      let used = exports_info.get_used_name(
        &context.compilation.exports_info_artifact,
        context.runtime,
        dep.names(),
      );

      let base = if dep.base().is_exports() {
        match module.get_exports_argument() {
          ExportsArgument::Exports => "exports".to_string(),
          ExportsArgument::RspackExports => context
            .runtime_template
            .render_runtime_variable(&RuntimeVariable::Exports),
        }
      } else if dep.base().is_module_exports() {
        let module_argument = match module.get_module_argument() {
          ModuleArgument::Module => "module".to_string(),
          ModuleArgument::RspackModule => context
            .runtime_template
            .render_runtime_variable(&RuntimeVariable::Module),
        };
        format!("{module_argument}.exports")
      } else if dep.base().is_this() {
        "this".to_string()
      } else {
        unreachable!();
      };

      let action = if dep.base().is_expression() {
        let content = match used {
          Some(UsedName::Normal(used)) => format!("{}{}", base, property_access(used, 0)),
          used => {
            let is_inlined = matches!(used, Some(UsedName::Inlined(_)));
            format!(
              "__webpack_{}_export__",
              if is_inlined { "inlined" } else { "unused" }
            )
          }
        };
        AstDependencyAction::expr(range, content.into_boxed_str())
      } else if dep.base().is_define_property() {
        let Some(value_range) = dep.value_range() else {
          return false;
        };
        let replacements = if let Some(UsedName::Normal(used)) = used {
          if used.is_empty() {
            return false;
          }
          vec![
            (
              format!(
                "Object.defineProperty({}{}, {}, (",
                base,
                property_access(used[0..used.len() - 1].iter(), 0),
                rspack_util::json_stringify_str(
                  used
                    .last()
                    .expect("Unexpected render define property base")
                    .as_str()
                )
              ),
              range.start,
              value_range.start,
            ),
            ("))".to_string(), value_range.end, range.end),
          ]
        } else {
          vec![
            (
              "__webpack_unused_export__ = (".to_string(),
              range.start,
              value_range.start,
            ),
            (")".to_string(), value_range.end, range.end),
          ]
        };
        AstDependencyAction::source_with_replacements(range, range, replacements)
      } else {
        return false;
      };
      let Some(action) = action else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<CommonJsExportRequireDependency>()
    {
      if !dep.base().is_expression() {
        return false;
      }

      let module_graph = context.compilation.get_module_graph();
      let module = module_graph
        .module_by_identifier(&context.module.identifier())
        .expect("should have mgm");

      let exports_info = context
        .compilation
        .exports_info_artifact
        .get_exports_info_data(&module.identifier());
      let used = exports_info.get_used_name(
        &context.compilation.exports_info_artifact,
        context.runtime,
        dep.names(),
      );

      let base = if dep.base().is_exports() {
        match module.get_exports_argument() {
          ExportsArgument::Exports => "exports".to_string(),
          ExportsArgument::RspackExports => context
            .runtime_template
            .render_runtime_variable(&RuntimeVariable::Exports),
        }
      } else if dep.base().is_module_exports() {
        let module_argument = match module.get_module_argument() {
          ModuleArgument::Module => "module".to_string(),
          ModuleArgument::RspackModule => context
            .runtime_template
            .render_runtime_variable(&RuntimeVariable::Module),
        };
        format!("{module_argument}.exports")
      } else if dep.base().is_this() {
        "this".to_string()
      } else {
        unreachable!();
      };

      let module_raw = if let Some(module_identifier) =
        module_graph.module_identifier_by_dependency_id(dep.id())
        && let Some(module_id) =
          ChunkGraph::get_module_id(&context.compilation.module_ids_artifact, *module_identifier)
      {
        format!(
          "{}({})",
          context
            .runtime_template
            .render_runtime_globals_without_adding(&RuntimeGlobals::REQUIRE),
          context
            .runtime_template
            .module_id_expr(dep.request(), module_id)
        )
      } else {
        context.runtime_template.missing_module(dep.request())
      };

      let ids = dep.get_ids(&module_graph);
      let require_expr = if let Some(imported_module) =
        module_graph.get_module_by_dependency_id(dep.id())
        && let Some(used_imported) = context
          .compilation
          .exports_info_artifact
          .get_exports_info_data(&imported_module.identifier())
          .get_used_name(
            &context.compilation.exports_info_artifact,
            context.runtime,
            ids,
          ) {
        match used_imported {
          UsedName::Normal(used_imported) => format!(
            "{}{}{}",
            module_raw,
            to_normal_comment(&property_access(ids, 0)),
            property_access(used_imported, 0)
          ),
          UsedName::Inlined(inlined) => inlined.render(&to_normal_comment(&format!(
            "inlined export {}",
            property_access(ids, 0)
          ))),
        }
      } else {
        module_raw
      };

      let expr = match used {
        Some(UsedName::Normal(used)) => {
          format!("{base}{} = {require_expr}", property_access(used, 0))
        }
        Some(UsedName::Inlined(_)) => format!("/* inlined reexport */ {require_expr}"),
        None => format!("/* unused reexport */ {require_expr}"),
      };

      let Some(action) = AstDependencyAction::expr(range, expr.into_boxed_str()) else {
        return false;
      };
      let Some(key) = key else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if dependency
      .as_any()
      .downcast_ref::<ImportDependency>()
      .is_some()
    {
      let Some(replacements) = self.render_ast_template_replacements(context, dependency, range)
      else {
        return false;
      };
      if !self.push_template_replacement_actions(plan, range, replacements) {
        return false;
      }
      let Some(key) = key else {
        return false;
      };
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if dependency
      .as_any()
      .downcast_ref::<ImportEagerDependency>()
      .is_some()
    {
      let Some(replacements) = self.render_ast_template_replacements(context, dependency, range)
      else {
        return false;
      };
      if !self.push_template_replacement_actions(plan, range, replacements) {
        return false;
      }
      let Some(key) = key else {
        return false;
      };
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if dependency
      .as_any()
      .downcast_ref::<ImportWeakDependency>()
      .is_some()
    {
      let Some(replacements) = self.render_ast_template_replacements(context, dependency, range)
      else {
        return false;
      };
      if !self.push_template_replacement_actions(plan, range, replacements) {
        return false;
      }
      let Some(key) = key else {
        return false;
      };
      plan.push_side_effect(AstDependencySideEffect::RenderTemplate(key));
      return true;
    }

    if dependency
      .as_any()
      .downcast_ref::<RequireMainDependency>()
      .is_some()
    {
      let content = format!(
        "{}[{}]",
        context
          .runtime_template
          .render_runtime_globals_without_adding(&RuntimeGlobals::MODULE_CACHE),
        context
          .runtime_template
          .render_runtime_globals_without_adding(&RuntimeGlobals::ENTRY_MODULE_ID)
      );
      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      let mut runtime_requirements = RuntimeGlobals::MODULE_CACHE;
      runtime_requirements.insert(RuntimeGlobals::ENTRY_MODULE_ID);
      plan.push_side_effect(AstDependencySideEffect::RuntimeRequirements(
        runtime_requirements,
      ));
      return true;
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<IsIncludeDependency>() {
      let included = context
        .compilation
        .get_module_graph()
        .connection_by_dependency_id(&dep.id)
        .is_some_and(|connection| {
          context
            .compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .get_number_of_module_chunks(*connection.module_identifier())
            > 0
        });
      let Some(action) = AstDependencyAction::expr(range, included.to_string().into_boxed_str())
      else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency.as_any().downcast_ref::<ExportInfoDependency>() {
      let value = dep
        .get_property(context)
        .unwrap_or_else(|| "undefined".to_string());
      let Some(action) = AstDependencyAction::expr(range, value.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      return true;
    }

    if let Some(dep) = dependency
      .as_any()
      .downcast_ref::<ModuleArgumentDependency>()
    {
      let module_argument = context
        .compilation
        .get_module_graph()
        .module_by_identifier(&context.module.identifier())
        .expect("should have mgm")
        .get_module_argument();
      let mut content = match module_argument {
        ModuleArgument::Module => "module".to_string(),
        ModuleArgument::RspackModule => context
          .runtime_template
          .render_runtime_variable(&RuntimeVariable::Module),
      };
      let mut runtime_requirements = RuntimeGlobals::MODULE;

      if let Some(id) = dep.id() {
        match id {
          "id" => runtime_requirements.insert(RuntimeGlobals::MODULE_ID),
          "loaded" => runtime_requirements.insert(RuntimeGlobals::MODULE_LOADED),
          _ => {}
        };
        content.push('.');
        content.push_str(id);
      }

      let Some(action) = AstDependencyAction::expr(range, content.into_boxed_str()) else {
        return false;
      };
      plan.push_action(action);
      plan.push_side_effect(AstDependencySideEffect::RuntimeRequirements(
        runtime_requirements,
      ));
      return true;
    }

    false
  }

  fn try_render_ast_dependencies(
    &self,
    source_text: &str,
    module: &dyn Module,
    context: &mut TemplateContext,
  ) -> Option<BoxSource> {
    let plan = self.collect_ast_render_plan(module, context)?;
    let source = render_ast_dependencies(source_text, module.module_type(), &plan)?;
    self.apply_ast_dependency_side_effects(module, context, &plan, source_text.len());
    Some(RawStringSource::from(source).boxed())
  }

  fn try_render_ast_dependencies_with_source_map(
    &self,
    source_text: &str,
    source: &BoxSource,
    module: &dyn Module,
    context: &mut TemplateContext,
  ) -> Option<BoxSource> {
    let plan = self.collect_ast_render_plan(module, context)?;
    let mut source = ReplaceSource::new(source.clone());
    if !apply_ast_dependency_replacements(source_text, module.module_type(), &plan, &mut source) {
      return None;
    }
    self.apply_ast_dependency_side_effects(module, context, &plan, source_text.len());
    Some(source.boxed())
  }

  fn apply_ast_dependency_side_effects(
    &self,
    module: &dyn Module,
    context: &mut TemplateContext,
    plan: &AstDependencyRenderPlan,
    source_len: usize,
  ) {
    let mut side_effect_source =
      ReplaceSource::new(RawStringSource::from(" ".repeat(source_len)).boxed());
    for side_effect in plan.side_effects() {
      match side_effect {
        AstDependencySideEffect::RenderTemplate(AstDependencyRenderKey::Dependency(
          dependency_id,
        )) => {
          self.source_dependency(
            context.compilation,
            dependency_id,
            &mut side_effect_source,
            context,
          );
        }
        AstDependencySideEffect::RenderTemplate(AstDependencyRenderKey::Presentational(idx)) => {
          if let Some(dependency) = module
            .get_presentational_dependencies()
            .and_then(|dependencies| dependencies.get(*idx))
          {
            self.render_presentational_dependency(
              context.compilation,
              dependency.as_ref(),
              &mut side_effect_source,
              context,
            );
          }
        }
        _ => side_effect.apply(context),
      }
    }
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

  fn source_ast_dependency(
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
        template.render_ast(dependency, source, context)
      } else {
        panic!(
          "Can not find dependency template of {:?}",
          dependency.dependency_template()
        );
      }
    }
  }

  fn render_presentational_dependency(
    &self,
    compilation: &Compilation,
    dependency: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
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

  fn render_ast_presentational_dependency(
    &self,
    compilation: &Compilation,
    dependency: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    if let Some(template) = dependency
      .dependency_template()
      .and_then(|template_type| compilation.get_dependency_template(template_type))
    {
      template.render_ast(dependency, source, context)
    } else {
      panic!(
        "Can not find dependency template of {:?}",
        dependency.dependency_template()
      );
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
      let source_text = source.source().into_string_lossy();
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

      let has_source_map = source
        .map(&ObjectPool::default(), &MapOptions::default())
        .is_some();
      if !has_source_map
        && let Some(source) = self.try_render_ast_dependencies(&source_text, module, &mut context)
      {
        generate_context.concatenation_scope = context.concatenation_scope.take();
        return render_init_fragments(source, init_fragments, generate_context);
      }
      if has_source_map
        && let Some(source) = self.try_render_ast_dependencies_with_source_map(
          &source_text,
          source,
          module,
          &mut context,
        )
      {
        generate_context.concatenation_scope = context.concatenation_scope.take();
        return render_init_fragments(source, init_fragments, generate_context);
      }

      let mut source = ReplaceSource::new(source.clone());
      let mut ast_dependencies = Vec::new();
      module.get_dependencies().iter().for_each(|dependency_id| {
        self.collect_ast_dependency(compilation, dependency_id, &mut ast_dependencies)
      });

      if let Some(dependencies) = module.get_presentational_dependencies() {
        dependencies
          .iter()
          .enumerate()
          .for_each(|(idx, dependency)| {
            if let Some(entry) = AstDependencyRenderEntry::new(
              AstDependencyRenderKey::Presentational(idx),
              dependency.as_ref(),
            ) {
              ast_dependencies.push(entry);
            }
          });
      };

      module
        .get_blocks()
        .iter()
        .for_each(|block_id| self.collect_ast_block(compilation, block_id, &mut ast_dependencies));

      let mut applied_ast_dependency_ids = HashSet::new();
      let mut applied_ast_presentational_dependencies = HashSet::new();
      if !ast_dependencies.is_empty()
        && mark_ast_dependencies(&source_text, module.module_type(), &mut ast_dependencies)
      {
        for entry in ast_dependencies.iter().filter(|entry| entry.applied()) {
          match entry.key {
            AstDependencyRenderKey::Dependency(dependency_id) => {
              applied_ast_dependency_ids.insert(dependency_id);
            }
            AstDependencyRenderKey::Presentational(idx) => {
              applied_ast_presentational_dependencies.insert(idx);
            }
          }
        }
      }

      module.get_dependencies().iter().for_each(|dependency_id| {
        if applied_ast_dependency_ids.contains(dependency_id) {
          self.source_ast_dependency(compilation, dependency_id, &mut source, &mut context)
        } else {
          self.source_dependency(compilation, dependency_id, &mut source, &mut context)
        }
      });

      if let Some(dependencies) = module.get_presentational_dependencies() {
        dependencies
          .iter()
          .enumerate()
          .for_each(|(idx, dependency)| {
            if applied_ast_presentational_dependencies.contains(&idx) {
              self.render_ast_presentational_dependency(
                compilation,
                dependency.as_ref(),
                &mut source,
                &mut context,
              )
            } else {
              self.render_presentational_dependency(
                compilation,
                dependency.as_ref(),
                &mut source,
                &mut context,
              );
            }
          });
      };

      module.get_blocks().iter().for_each(|block_id| {
        self.source_block(
          compilation,
          block_id,
          &mut source,
          &mut context,
          &applied_ast_dependency_ids,
        )
      });
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

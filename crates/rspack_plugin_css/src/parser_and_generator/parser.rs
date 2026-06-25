use std::{path::Path, sync::Arc};

use once_cell::sync::OnceCell;
use rspack_core::{
  BoxDependencyTemplate, BoxModuleDependency, ConstDependency, CssAutoOrModuleParserOptions,
  CssExport, CssExports, CssExportsConvention, CssLayer, CssLocalNames, CssModuleGeneratorOptions,
  CssModuleRenderCondition, CssParserImport, CssParserImportContext, Dependency, DependencyId,
  DependencyRange, ModuleType, ParseContext, ParseResult, ResourceData,
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics, remove_bom, rspack_sources::Source,
  topological_sort,
};
use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, Severity, TWithDiagnosticArray};
use rustc_hash::{FxHashMap, FxHashSet};
use smol_str::SmolStr;

use super::{REGEX_CUSTOM_PROPERTY_IDENT, REGEX_IS_COMMENTS, REGEX_IS_MODULES};
use crate::{
  dependency::{
    CssComposeDependency, CssExportDependency, CssIcssSymbolDependency, CssIcssSymbolValue,
    CssImportDependency, CssLocalIdentDependency, CssSelfReferenceLocalIdentDependency,
    CssSelfReferenceLocalIdentReplacement, CssUrlDependency,
  },
  parser_and_generator::generator::update_css_exports,
  utils::{
    LocalIdentModuleHashOptions, LocalIdentOptions, PresentationalDependencyHashUpdate,
    css_parsing_traceable_error, export_locals_convention, normalize_url,
    replace_module_request_prefix, unescape,
  },
};

pub(super) struct CssModuleParser<'context> {
  generator_options: &'context CssModuleGeneratorOptions,
  parser_options: &'context CssAutoOrModuleParserOptions,
  exports_only: bool,
  parse_context: ParseContext<'context>,
  source: Arc<dyn Source>,
  source_code: Arc<str>,
  diagnostics: Vec<Diagnostic>,
  dependencies: Vec<Box<dyn Dependency>>,
  presentational_dependencies: Vec<BoxDependencyTemplate>,
  code_generation_dependencies: Vec<BoxModuleDependency>,
  css_exports: CssExports,
  css_local_names: CssLocalNames,
  inherited_render_conditions: Vec<CssModuleRenderCondition>,
  render_condition: CssModuleRenderCondition,
  icss_definitions: FxHashMap<String, IcssDefinition>,
  current_icss_import_from: Option<String>,
  composes_order: ComposesOrderState,
  local_ident_options: OnceCell<LocalIdentOptions<'context>>,
}

#[derive(Debug, Clone)]
enum IcssDefinition {
  Value(String),
  Import {
    import_name: String,
    request: String,
  },
}

#[derive(Default)]
struct ComposesOrderState {
  graph: FxHashMap<DependencyId, FxHashSet<DependencyId>>,
  request_to_dependency: FxHashMap<String, DependencyId>,
  dependencies_in_source_order: Vec<(DependencyId, i32)>,
  compose_dependency_count: usize,
  current_rule_key: Option<String>,
  current_rule_prev_dependency: Option<DependencyId>,
  current_rule_dependencies: FxHashSet<DependencyId>,
}

impl ComposesOrderState {
  fn reset_current_rule(&mut self) {
    self.current_rule_key = None;
    self.current_rule_prev_dependency = None;
    self.current_rule_dependencies.clear();
  }

  fn track_request_order(
    &mut self,
    local_classes: &[String],
    request: &str,
    source_start: u32,
    dependency_id: DependencyId,
  ) {
    let rule_key = local_classes.join("\0");
    if self.current_rule_key.as_deref() != Some(rule_key.as_str()) {
      self.current_rule_key = Some(rule_key);
      self.current_rule_prev_dependency = None;
      self.current_rule_dependencies.clear();
    }

    let dependency_id = self.dependency_id_for_request(request, source_start, dependency_id);
    if !self.current_rule_dependencies.insert(dependency_id) {
      return;
    }

    if let Some(prev_dependency_id) = self.current_rule_prev_dependency
      && prev_dependency_id != dependency_id
    {
      self
        .graph
        .entry(prev_dependency_id)
        .or_default()
        .insert(dependency_id);
    }

    self.current_rule_prev_dependency = Some(dependency_id);
  }

  fn dependency_id_for_request(
    &mut self,
    request: &str,
    source_start: u32,
    dependency_id: DependencyId,
  ) -> DependencyId {
    if let Some(dependency_id) = self.request_to_dependency.get(request) {
      return *dependency_id;
    }

    self
      .request_to_dependency
      .insert(request.to_string(), dependency_id);
    self
      .dependencies_in_source_order
      .push((dependency_id, source_order_to_i32(source_start)));
    self.compose_dependency_count += 1;
    dependency_id
  }

  fn has_multiple_dependencies(&self) -> bool {
    self.compose_dependency_count > 1
  }

  fn source_order(&self) -> Vec<(DependencyId, i32)> {
    let mut dependencies_in_source_order = self.dependencies_in_source_order.clone();
    dependencies_in_source_order.sort_by_key(|(_, source_order)| *source_order);
    let base_source_order = dependencies_in_source_order
      .first()
      .map(|(_, source_order)| *source_order)
      .unwrap_or_default();
    let dependencies_in_source_order = dependencies_in_source_order
      .into_iter()
      .map(|(dependency_id, _)| dependency_id)
      .collect::<Vec<_>>();

    topological_sort(dependencies_in_source_order, |dependency_id| {
      self
        .graph
        .get(&dependency_id)
        .into_iter()
        .flat_map(|successors| successors.iter().copied())
        .collect::<Vec<_>>()
    })
    .into_iter()
    .enumerate()
    .map(|(source_order, dependency_id)| {
      (
        dependency_id,
        base_source_order.saturating_add(source_order.try_into().unwrap_or(i32::MAX)),
      )
    })
    .collect()
  }
}

#[derive(Default)]
struct LocalCssIdentDeclarations {
  keyframes: FxHashSet<SmolStr>,
  custom_idents: FxHashSet<SmolStr>,
  containers: FxHashSet<SmolStr>,
  functions: FxHashSet<SmolStr>,
  grids: FxHashSet<SmolStr>,
  vars: FxHashSet<SmolStr>,
}

impl LocalCssIdentDeclarations {
  fn has_keyframes(&self, name: &str) -> bool {
    self.keyframes.contains(&normalize_ident_name(name))
  }

  fn has_custom_ident(&self, name: &str) -> bool {
    self.custom_idents.contains(&normalize_ident_name(name))
  }

  fn has_container(&self, name: &str) -> bool {
    self.containers.contains(&normalize_ident_name(name))
  }

  fn has_function(&self, name: &str) -> bool {
    self.functions.contains(&normalize_ident_name(name))
  }

  fn has_grid(&self, name: &str) -> bool {
    self.grids.contains(&normalize_ident_name(name))
  }

  fn has_var(&self, name: &str) -> bool {
    self.vars.contains(&normalize_dashed_ident_name(name))
  }
}

fn source_order_to_i32(source_order: u32) -> i32 {
  source_order.try_into().unwrap_or(i32::MAX)
}

fn is_custom_property_name(value: &str) -> bool {
  !value.is_empty()
    && value
      .bytes()
      .all(|c| !c.is_ascii_whitespace() && !matches!(c, b')' | b'(' | b'"' | b'\''))
}

fn normalize_ident_name(name: &str) -> SmolStr {
  SmolStr::new(unescape(name).as_ref())
}

fn normalize_dashed_ident_name(name: &str) -> SmolStr {
  SmolStr::new(unescape(name).trim_start_matches("--"))
}

impl<'context> CssModuleParser<'context> {
  pub fn new(
    generator_options: &'context CssModuleGeneratorOptions,
    parser_options: &'context CssAutoOrModuleParserOptions,
    exports_only: bool,
    parse_context: ParseContext<'context>,
  ) -> Self {
    let source = remove_bom(parse_context.source.clone());
    let source_code: Arc<str> = source.source().into_string_lossy().into();
    let (inherited_render_conditions, render_condition) = parse_context
      .build_info
      .css
      .as_deref()
      .map(|css| {
        (
          css.inherited_render_conditions.clone(),
          css.render_condition.clone(),
        )
      })
      .unwrap_or_default();

    Self {
      generator_options,
      parser_options,
      exports_only,
      parse_context,
      source,
      source_code,
      diagnostics: vec![],
      dependencies: vec![],
      presentational_dependencies: vec![],
      code_generation_dependencies: vec![],
      css_exports: Default::default(),
      css_local_names: Default::default(),
      inherited_render_conditions,
      render_condition,
      icss_definitions: Default::default(),
      current_icss_import_from: None,
      composes_order: Default::default(),
      local_ident_options: OnceCell::new(),
    }
  }

  pub async fn parse(mut self) -> Result<TWithDiagnosticArray<ParseResult>> {
    let mode = self.mode();
    let deps_source_code = self.source_code.clone();
    let (deps, warnings) = css_module_lexer::collect_dependencies(&deps_source_code, mode);
    let local_css_ident_declarations = self.collect_local_css_ident_declarations(&deps);
    let module_hash_options = self.create_module_hash_options(&deps, &local_css_ident_declarations);

    for dependency in deps {
      self
        .handle_dependency(
          dependency,
          &module_hash_options,
          &local_css_ident_declarations,
        )
        .await?;
    }

    self.apply_composes_source_order();
    self.add_warnings(warnings);

    let css_build_info = self.parse_context.build_info.css.get_or_insert_default();
    css_build_info.exports = self.css_exports;
    css_build_info.local_names = self.css_local_names;

    Ok(
      ParseResult {
        dependencies: self.dependencies,
        blocks: vec![],
        presentational_dependencies: self.presentational_dependencies,
        code_generation_dependencies: self.code_generation_dependencies,
        source: self.source,
        side_effects_bailout: None,
      }
      .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
        self.diagnostics,
        self.parse_context.loaders,
      )),
    )
  }

  fn mode(&self) -> css_module_lexer::Mode {
    let resource_path = self.resource_data().path();
    match self.parse_context.module_type {
      ModuleType::CssModule if self.pure() => css_module_lexer::Mode::Pure,
      ModuleType::CssModule => css_module_lexer::Mode::Local,
      ModuleType::CssGlobal => css_module_lexer::Mode::Global,
      ModuleType::CssAuto
        if resource_path.is_some()
          && REGEX_IS_MODULES.is_match(
            resource_path
              .expect("should have resource_path for module_type css/auto")
              .as_str(),
          ) =>
      {
        if self.pure() {
          css_module_lexer::Mode::Pure
        } else {
          css_module_lexer::Mode::Local
        }
      }
      _ => css_module_lexer::Mode::Css,
    }
  }

  fn create_module_hash_options<'source>(
    &self,
    deps: &[css_module_lexer::Dependency<'source>],
    local_css_ident_declarations: &LocalCssIdentDeclarations,
  ) -> LocalIdentModuleHashOptions<'source> {
    let mut export_dependency_names = Vec::new();
    let mut graph_export_name_set = FxHashSet::default();
    let mut presentational_dependency_hash_updates = Vec::new();
    let convention = self.generator_options.exports_convention;

    for dependency in deps.iter() {
      match dependency {
        css_module_lexer::Dependency::LocalClass { name, .. }
        | css_module_lexer::Dependency::LocalId { name, .. } => {
          if let Some(convention) = convention {
            let (_prefix, name) = name.split_at(1);
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalKeyframes { name, .. }
          if self.animation() && local_css_ident_declarations.has_keyframes(name) =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalKeyframesDecl { name, .. } if self.animation() => {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalCounterStyle { name, .. }
        | css_module_lexer::Dependency::LocalFontPalette { name, .. }
          if self.custom_idents() && local_css_ident_declarations.has_custom_ident(name) =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalCounterStyleDecl { name, .. }
        | css_module_lexer::Dependency::LocalFontPaletteDecl { name, .. }
          if self.custom_idents() =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalContainer { name, .. }
          if self.container() && local_css_ident_declarations.has_container(name) =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalContainerDecl { name, .. } if self.container() => {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalFunction { name, .. }
          if self.function() && local_css_ident_declarations.has_function(name) =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalFunctionDecl { name, .. } if self.function() => {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalGrid { name, .. }
          if self.grid() && local_css_ident_declarations.has_grid(name) =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalGridDecl { name, .. } if self.grid() => {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalContainer { name, .. }
          if self.container() && local_css_ident_declarations.has_container(name) =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalContainerDecl { name, .. } if self.container() => {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalFunction { name, .. }
          if self.function() && local_css_ident_declarations.has_function(name) =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalFunctionDecl { name, .. } if self.function() => {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalGrid { name, .. }
          if self.grid() && local_css_ident_declarations.has_grid(name) =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalGridDecl { name, .. } if self.grid() => {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalVar { name, from, .. }
          if self.dashed_idents()
            && self.should_handle_local_var_usage(name, *from, local_css_ident_declarations) =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::LocalVarDecl { name, .. }
        | css_module_lexer::Dependency::LocalPropertyDecl { name, .. }
          if self.dashed_idents() =>
        {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::ICSSExportValue { prop: name, .. } => {
          if let Some(convention) = convention {
            self.collect_export_dependency_name(
              unescape(name).into_owned(),
              convention,
              &mut export_dependency_names,
              &mut graph_export_name_set,
            );
          }
        }
        css_module_lexer::Dependency::Replace { content, range } => {
          presentational_dependency_hash_updates.push(PresentationalDependencyHashUpdate {
            start: range.start,
            end: range.end + 1,
            content,
          });
        }
        _ => {}
      }
    }

    if self.dashed_idents()
      && let Some(convention) = convention
    {
      for captures in REGEX_CUSTOM_PROPERTY_IDENT.captures_iter(&self.source_code) {
        if let Some(name) = captures.get(2) {
          let name = name.as_str();
          if !local_css_ident_declarations.has_var(name) {
            continue;
          }
          self.collect_export_dependency_name(
            name.to_string(),
            convention,
            &mut export_dependency_names,
            &mut graph_export_name_set,
          );
        }
      }
    }

    LocalIdentModuleHashOptions {
      export_dependency_names,
      graph_export_names: graph_export_name_set,
      presentational_dependency_hash_updates,
      exports_only: self.exports_only,
      es_module: self.es_module(),
      named_exports: self.named_exports(),
      exports_convention: self.generator_options.exports_convention,
    }
  }

  fn collect_export_dependency_name(
    &self,
    name: String,
    convention: rspack_core::CssExportsConvention,
    export_dependency_names: &mut Vec<String>,
    graph_export_name_set: &mut FxHashSet<String>,
  ) {
    for convention_name in export_locals_convention(&name, convention) {
      graph_export_name_set.insert(convention_name);
    }
    export_dependency_names.push(name);
  }

  fn collect_local_css_ident_declarations<'source>(
    &self,
    deps: &[css_module_lexer::Dependency<'source>],
  ) -> LocalCssIdentDeclarations {
    let mut declarations = LocalCssIdentDeclarations::default();

    if !self.animation()
      && !self.custom_idents()
      && !self.container()
      && !self.function()
      && !self.grid()
      && !self.dashed_idents()
    {
      return declarations;
    }

    for dependency in deps {
      match dependency {
        css_module_lexer::Dependency::LocalKeyframesDecl { name, .. } if self.animation() => {
          declarations.keyframes.insert(normalize_ident_name(name));
        }
        css_module_lexer::Dependency::LocalCounterStyleDecl { name, .. }
        | css_module_lexer::Dependency::LocalFontPaletteDecl { name, .. }
          if self.custom_idents() =>
        {
          declarations
            .custom_idents
            .insert(normalize_ident_name(name));
        }
        css_module_lexer::Dependency::LocalContainerDecl { name, .. } if self.container() => {
          declarations.containers.insert(normalize_ident_name(name));
        }
        css_module_lexer::Dependency::LocalFunctionDecl { name, .. } if self.function() => {
          declarations.functions.insert(normalize_ident_name(name));
        }
        css_module_lexer::Dependency::LocalGridDecl { name, .. } if self.grid() => {
          declarations.grids.insert(normalize_ident_name(name));
        }
        css_module_lexer::Dependency::LocalVarDecl { name, .. }
        | css_module_lexer::Dependency::LocalPropertyDecl { name, .. }
          if self.dashed_idents() =>
        {
          declarations.vars.insert(normalize_dashed_ident_name(name));
        }
        css_module_lexer::Dependency::ICSSExportValue { prop, .. }
        | css_module_lexer::Dependency::ICSSImportValue { prop, .. }
          if self.dashed_idents()
            && prop.strip_prefix("--").is_some_and(is_custom_property_name) =>
        {
          declarations.vars.insert(normalize_dashed_ident_name(prop));
        }
        _ => {}
      }
    }

    declarations
  }

  fn should_handle_local_var_usage(
    &self,
    name: &str,
    from: Option<&str>,
    local_css_ident_declarations: &LocalCssIdentDeclarations,
  ) -> bool {
    if let Some(from) = from {
      return from.trim_matches(|c| c == '\'' || c == '"') != "global";
    }

    local_css_ident_declarations.has_var(name)
  }

  fn presentational_replace_range(
    &self,
    content: &str,
    range: css_module_lexer::Range,
  ) -> DependencyRange {
    if !content.is_empty() {
      return (range.start, range.end).into();
    }

    let source = self.source_code.as_ref();
    let mut start = range.start as usize;
    let mut end = range.end as usize;
    let bytes = source.as_bytes();
    let line_start = bytes[..start]
      .iter()
      .rposition(|byte| *byte == b'\n')
      .map_or(0, |pos| pos + 1);

    if bytes[line_start..start]
      .iter()
      .all(|byte| *byte == b' ' || *byte == b'\t')
    {
      start = line_start;
      if end < bytes.len() && bytes[end] == b'\r' {
        end += 1;
      }
      if end < bytes.len() && bytes[end] == b'\n' {
        end += 1;
      }
    }

    (start as u32, end as u32).into()
  }

  async fn handle_dependency<'source>(
    &mut self,
    dependency: css_module_lexer::Dependency<'source>,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
    local_css_ident_declarations: &LocalCssIdentDeclarations,
  ) -> Result<()> {
    match dependency {
      css_module_lexer::Dependency::Url {
        request,
        range,
        kind,
      } => self.handle_url(request, range, kind),
      css_module_lexer::Dependency::Import {
        request,
        range,
        media,
        supports,
        layer,
      } => {
        self
          .handle_import(request, range, media, supports, layer)
          .await
      }
      css_module_lexer::Dependency::Replace { content, range } => {
        let range = self.presentational_replace_range(content, range);
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(range, content.into())));
        Ok(())
      }
      css_module_lexer::Dependency::LocalClass { name, range, .. }
      | css_module_lexer::Dependency::LocalId { name, range, .. } => {
        self.reset_current_composes_rule();
        let (_prefix, name) = name.split_at(1);
        self
          .handle_local_ident_declaration(name, range.start + 1, range.end, module_hash_options)
          .await
      }
      css_module_lexer::Dependency::LocalKeyframes { name, range, .. } => {
        self
          .handle_optional_local_ident_usage(
            self.animation() && local_css_ident_declarations.has_keyframes(name),
            name,
            range,
            module_hash_options,
          )
          .await
      }
      css_module_lexer::Dependency::LocalKeyframesDecl { name, range, .. } => {
        self
          .handle_optional_local_ident_declaration(
            self.animation(),
            name,
            range,
            module_hash_options,
          )
          .await
      }
      css_module_lexer::Dependency::Composes {
        local_classes,
        names,
        from,
        range,
      } => {
        self.handle_composes(local_classes, names, from, range);
        Ok(())
      }
      css_module_lexer::Dependency::ICSSExportValue { prop, value } => {
        self.handle_icss_export_value(prop, value);
        Ok(())
      }
      css_module_lexer::Dependency::ICSSImportFrom { path } => {
        self.handle_icss_import_from(path);
        Ok(())
      }
      css_module_lexer::Dependency::ICSSImportValue { prop, value } => {
        self.handle_icss_import_value(prop, value);
        Ok(())
      }
      css_module_lexer::Dependency::ICSSSymbol { name, range } => {
        self.handle_icss_symbol(name, range);
        Ok(())
      }
      css_module_lexer::Dependency::LocalCounterStyle { name, range, .. }
      | css_module_lexer::Dependency::LocalFontPalette { name, range, .. } => {
        self
          .handle_optional_local_ident_usage(
            self.custom_idents() && local_css_ident_declarations.has_custom_ident(name),
            name,
            range,
            module_hash_options,
          )
          .await
      }
      css_module_lexer::Dependency::LocalCounterStyleDecl { name, range, .. }
      | css_module_lexer::Dependency::LocalFontPaletteDecl { name, range, .. } => {
        self
          .handle_optional_local_ident_declaration(
            self.custom_idents(),
            name,
            range,
            module_hash_options,
          )
          .await
      }
      css_module_lexer::Dependency::LocalContainer { name, range, .. } => {
        self
          .handle_optional_local_ident_usage(
            self.container() && local_css_ident_declarations.has_container(name),
            name,
            range,
            module_hash_options,
          )
          .await
      }
      css_module_lexer::Dependency::LocalContainerDecl { name, range, .. } => {
        self
          .handle_optional_local_ident_declaration(
            self.container(),
            name,
            range,
            module_hash_options,
          )
          .await
      }
      css_module_lexer::Dependency::LocalFunction { name, range, .. } => {
        self
          .handle_optional_local_dashed_ident_usage(
            self.function() && local_css_ident_declarations.has_function(name),
            name,
            range,
            module_hash_options,
          )
          .await
      }
      css_module_lexer::Dependency::LocalFunctionDecl { name, range, .. } => {
        self
          .handle_optional_local_dashed_ident_declaration(
            self.function(),
            name,
            range,
            module_hash_options,
          )
          .await
      }
      css_module_lexer::Dependency::LocalGrid { name, range, .. } => {
        self
          .handle_optional_local_ident_usage(
            self.grid() && local_css_ident_declarations.has_grid(name),
            name,
            range,
            module_hash_options,
          )
          .await
      }
      css_module_lexer::Dependency::LocalGridDecl { name, range, .. } => {
        self
          .handle_optional_local_ident_declaration(self.grid(), name, range, module_hash_options)
          .await
      }
      css_module_lexer::Dependency::LocalVar { name, range, from } => {
        self
          .handle_optional_local_var_usage(
            self.dashed_idents(),
            name,
            range,
            from,
            module_hash_options,
            local_css_ident_declarations,
          )
          .await
      }
      css_module_lexer::Dependency::LocalVarDecl { name, range, .. }
      | css_module_lexer::Dependency::LocalPropertyDecl { name, range, .. } => {
        self
          .handle_optional_local_var_declaration(
            self.dashed_idents(),
            name,
            range,
            module_hash_options,
          )
          .await
      }
      _ => Ok(()),
    }
  }

  fn handle_url(
    &mut self,
    request: &str,
    range: css_module_lexer::Range,
    kind: css_module_lexer::UrlRangeKind,
  ) -> Result<()> {
    if request.trim().is_empty() || !self.url() {
      return Ok(());
    }

    let request = replace_module_request_prefix(
      request,
      &mut self.diagnostics,
      &self.source_code,
      range.start,
      range.end,
    );
    let request = normalize_url(request);
    let dep = Box::new(CssUrlDependency::new(
      request,
      DependencyRange::new(range.start, range.end),
      matches!(kind, css_module_lexer::UrlRangeKind::Function),
    ));
    self.dependencies.push(dep.clone());
    self.code_generation_dependencies.push(dep);
    Ok(())
  }

  async fn handle_import(
    &mut self,
    request: &str,
    range: css_module_lexer::Range,
    media: Option<&str>,
    supports: Option<&str>,
    layer: Option<&str>,
  ) -> Result<()> {
    let request = normalize_url(request);
    if request.trim().is_empty() {
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          (range.start, range.end).into(),
          "".into(),
        )));
      return Ok(());
    }

    if !self.import() || !self.should_import(&request, media, supports, layer).await {
      return Ok(());
    }

    let request = replace_module_request_prefix(
      &request,
      &mut self.diagnostics,
      &self.source_code,
      range.start,
      range.end,
    )
    .to_string();
    let layer = layer.map(str::trim).map(|s| {
      if s.is_empty() {
        CssLayer::Anonymous
      } else {
        CssLayer::Named(s.into())
      }
    });
    let inherited_render_conditions = self.css_import_inherited_render_conditions();
    let render_condition = CssModuleRenderCondition::new(
      media.map(|media| media.trim().into()),
      supports.map(|supports| supports.trim().into()),
      layer,
    );

    self.dependencies.push(Box::new(CssImportDependency::new(
      request,
      DependencyRange::new(range.start, range.end),
      inherited_render_conditions,
      render_condition,
    )));
    Ok(())
  }

  fn css_import_inherited_render_conditions(&self) -> Vec<CssModuleRenderCondition> {
    if self.render_condition.is_empty() {
      return self.inherited_render_conditions.clone();
    }

    let mut inherited_render_conditions =
      Vec::with_capacity(self.inherited_render_conditions.len() + 1);
    inherited_render_conditions.extend(self.inherited_render_conditions.iter().cloned());
    inherited_render_conditions.push(self.render_condition.clone());
    inherited_render_conditions
  }

  async fn should_import(
    &self,
    request: &str,
    media: Option<&str>,
    supports: Option<&str>,
    layer: Option<&str>,
  ) -> bool {
    match self.resolve_import() {
      CssParserImport::Bool(b) => *b,
      CssParserImport::Func(f) => {
        let args = CssParserImportContext {
          url: request.to_string(),
          media: media.map(|s| s.to_string()),
          resource_path: self
            .resource_data()
            .path()
            .map(|p| p.as_str().to_string())
            .unwrap_or_default(),
          supports: supports.map(|s| s.to_string()),
          layer: layer.map(|s| s.to_string()),
        };
        (f(args).await).unwrap_or(true)
      }
    }
  }

  fn resolve_import(&self) -> &CssParserImport {
    self
      .parser_options
      .resolve_import
      .as_ref()
      .unwrap_or(&CssParserImport::Bool(true))
  }

  fn url(&self) -> bool {
    self.parser_options.url.expect("should have url")
  }

  fn import(&self) -> bool {
    self.parser_options.r#import.unwrap_or(true)
  }

  fn animation(&self) -> bool {
    self.parser_options.animation.unwrap_or(true)
  }

  fn container(&self) -> bool {
    self
      .parser_options
      .container
      .expect("should have container")
  }

  fn custom_idents(&self) -> bool {
    self
      .parser_options
      .custom_idents
      .expect("should have custom_idents")
  }

  fn dashed_idents(&self) -> bool {
    self
      .parser_options
      .dashed_idents
      .expect("should have dashed_idents")
  }

  fn function(&self) -> bool {
    self
      .parser_options
      .r#function
      .expect("should have function")
  }

  fn grid(&self) -> bool {
    self.parser_options.grid.expect("should have grid")
  }

  fn pure(&self) -> bool {
    self.parser_options.pure.unwrap_or(false)
  }

  fn convention(&self) -> CssExportsConvention {
    self
      .generator_options
      .exports_convention
      .expect("should have convention for module_type css/auto, css/global or css/module")
  }

  fn named_exports(&self) -> bool {
    self
      .parser_options
      .named_exports
      .expect("should have named_exports")
  }

  fn es_module(&self) -> bool {
    self
      .generator_options
      .es_module
      .expect("should have es_module")
  }

  fn get_local_ident_options(&self) -> &LocalIdentOptions<'_> {
    self.local_ident_options.get_or_init(move || {
      LocalIdentOptions::new(
        self.resource_data(),
        self.parse_context.module_type,
        self.source_code.clone(),
        self.parse_context.compiler_options,
        self.generator_options,
      )
    })
  }

  async fn handle_local_ident_usage(
    &mut self,
    name: &str,
    range: css_module_lexer::Range,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<()> {
    let name = unescape(name);
    let (local_ident, convention_names) = self
      .resolve_local_ident_and_update_exports(&name, module_hash_options)
      .await?;
    self
      .dependencies
      .push(Box::new(CssSelfReferenceLocalIdentDependency::new(
        convention_names,
        vec![CssSelfReferenceLocalIdentReplacement {
          local_ident,
          range: (range.start, range.end).into(),
        }],
      )));
    Ok(())
  }

  async fn handle_optional_local_ident_usage(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    self
      .handle_local_ident_usage(name, range, module_hash_options)
      .await
  }

  async fn handle_local_ident_declaration(
    &mut self,
    name: &str,
    start: u32,
    end: u32,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<()> {
    let name = unescape(name);
    let (local_ident, convention_names) = self
      .resolve_local_ident_and_update_exports(&name, module_hash_options)
      .await?;

    self
      .css_local_names
      .insert(name.as_ref().into(), local_ident.as_str().into());

    self
      .dependencies
      .push(Box::new(CssLocalIdentDependency::new(
        local_ident,
        convention_names,
        start,
        end,
      )));
    Ok(())
  }

  async fn handle_local_var_usage(
    &mut self,
    name: &str,
    range: css_module_lexer::Range,
    from: Option<&str>,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
    local_css_ident_declarations: &LocalCssIdentDeclarations,
  ) -> Result<()> {
    let name = unescape(name);
    let name = name.trim_start_matches("--");

    if let Some(from) = from
      && from.trim_matches(|c| c == '\'' || c == '"') == "global"
    {
      return Ok(());
    }

    if from.is_none() && !local_css_ident_declarations.vars.contains(name) {
      return Ok(());
    }

    self
      .add_local_var_self_reference(name, range, module_hash_options)
      .await?;
    Ok(())
  }

  async fn add_local_var_self_reference(
    &mut self,
    name: &str,
    range: css_module_lexer::Range,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<()> {
    let (local_ident, convention_names) = self
      .resolve_local_var_ident_and_update_exports(name, module_hash_options)
      .await?;
    self
      .dependencies
      .push(Box::new(CssSelfReferenceLocalIdentDependency::new(
        convention_names,
        vec![CssSelfReferenceLocalIdentReplacement {
          local_ident,
          range: (range.start, range.end).into(),
        }],
      )));
    Ok(())
  }

  async fn handle_optional_local_var_usage(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
    from: Option<&str>,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
    local_css_ident_declarations: &LocalCssIdentDeclarations,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    self
      .handle_local_var_usage(
        name,
        range,
        from,
        module_hash_options,
        local_css_ident_declarations,
      )
      .await
  }

  async fn handle_optional_local_dashed_ident_usage(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    self
      .add_local_var_self_reference(name, range, module_hash_options)
      .await
  }

  async fn handle_local_var_declaration(
    &mut self,
    name: &str,
    start: u32,
    end: u32,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<()> {
    let name = unescape(name);
    let name = name.trim_start_matches("--");
    let (local_ident, convention_names) = self
      .resolve_local_var_ident_and_update_exports(name, module_hash_options)
      .await?;

    self
      .css_local_names
      .insert(name.into(), local_ident.as_str().into());

    self
      .dependencies
      .push(Box::new(CssLocalIdentDependency::new(
        local_ident,
        convention_names,
        start,
        end,
      )));
    Ok(())
  }

  async fn handle_optional_local_var_declaration(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    self
      .handle_local_var_declaration(name, range.start, range.end, module_hash_options)
      .await
  }

  async fn handle_optional_local_dashed_ident_declaration(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    self
      .handle_local_var_declaration(name, range.start, range.end, module_hash_options)
      .await
  }

  async fn handle_optional_local_ident_declaration(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    self
      .handle_local_ident_declaration(name, range.start, range.end, module_hash_options)
      .await
  }

  async fn resolve_local_ident_and_update_exports(
    &mut self,
    name: &str,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<(String, Vec<String>)> {
    let local_ident = {
      let local_ident_options = self.get_local_ident_options();
      local_ident_options
        .get_local_ident(name, module_hash_options)
        .await?
    };
    let convention = self.convention();
    let convention_names = export_locals_convention(name, convention);
    for convention_name in convention_names.iter() {
      if self.has_custom_property_export(convention_name) {
        continue;
      }
      update_css_exports(
        &mut self.css_exports,
        convention_name,
        CssExport {
          ident: local_ident.as_str().into(),
          orig_name: name.into(),
          from: None,
          id: None,
        },
      );
    }
    Ok((local_ident, convention_names))
  }

  async fn resolve_local_var_ident_and_update_exports(
    &mut self,
    name: &str,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
  ) -> Result<(String, Vec<String>)> {
    let local_ident = {
      let local_ident_options = self.get_local_ident_options();
      local_ident_options
        .get_local_ident(name, module_hash_options)
        .await?
    };
    let local_ident = local_ident
      .strip_prefix("_--")
      .unwrap_or(local_ident.as_str())
      .to_string();
    let local_ident = format!("--{local_ident}");
    let convention = self.convention();
    let convention_names = export_locals_convention(name, convention);
    for convention_name in convention_names.iter() {
      update_css_exports(
        &mut self.css_exports,
        convention_name,
        CssExport {
          ident: local_ident.as_str().into(),
          orig_name: name.into(),
          from: None,
          id: None,
        },
      );
    }
    Ok((local_ident, convention_names))
  }

  fn has_custom_property_export(&self, name: &str) -> bool {
    self.css_exports.get(name).is_some_and(|exports| {
      exports.iter().any(|export| {
        export.ident.starts_with("--")
          || export
            .ident
            .strip_prefix("_--")
            .is_some_and(is_custom_property_name)
      })
    })
  }

  fn handle_composes<'source>(
    &mut self,
    local_classes: impl IntoIterator<Item = &'source str>,
    names: impl IntoIterator<Item = &'source str>,
    from: Option<&'source str>,
    range: css_module_lexer::Range,
  ) {
    let local_classes = local_classes
      .into_iter()
      .map(|s| unescape(s).to_string())
      .collect::<Vec<_>>();
    let names = names
      .into_iter()
      .map(|s| unescape(s).to_string())
      .collect::<Vec<_>>();

    let mut dep_id = None;
    if let Some(from) = from
      && from != "global"
    {
      let from = from.trim_matches(|c| c == '\'' || c == '"');
      let dep = CssComposeDependency::new(
        from.to_string(),
        names.iter().map(|s| s.to_owned().into()).collect(),
        DependencyRange::new(range.start, range.end),
      );
      dep_id = Some(*dep.id());
      self
        .composes_order
        .track_request_order(&local_classes, from, range.start, *dep.id());
      self.dependencies.push(Box::new(dep));
    } else if from.is_none() {
      self
        .dependencies
        .push(Box::new(CssSelfReferenceLocalIdentDependency::new(
          names.clone(),
          vec![],
        )));
    }

    let convention = self.convention();
    for name in names {
      for local_class in local_classes.iter() {
        let convention_names = export_locals_convention(&name, convention);
        let convention_local_class = export_locals_convention(local_class, convention);

        for (convention_name, local_class) in
          convention_names.into_iter().zip(convention_local_class)
        {
          if from.is_none() {
            if let Some(existing) = self.get_icss_composes_exports(name.as_str()) {
              self
                .css_exports
                .get_mut(local_class.as_str())
                .expect("composes local class must already added to exports")
                .extend(existing);
              continue;
            }

            let existing = self.css_exports.get(name.as_str()).cloned();
            if let Some(existing) = existing {
              self
                .css_exports
                .get_mut(local_class.as_str())
                .expect("composes local class must already added to exports")
                .extend(existing);
              continue;
            }
          }

          self
            .css_exports
            .get_mut(local_class.as_str())
            .expect("composes local class must already added to exports")
            .insert(CssExport {
              ident: convention_name.as_str().into(),
              orig_name: name.as_str().into(),
              from: from
                .filter(|f| *f != "global")
                .map(|f| f.trim_matches(|c| c == '\'' || c == '"').into()),
              id: dep_id,
            });
        }
      }
    }
  }

  fn reset_current_composes_rule(&mut self) {
    self.composes_order.reset_current_rule();
  }

  fn apply_composes_source_order(&mut self) {
    if !self.composes_order.has_multiple_dependencies() {
      return;
    }

    let source_order_by_dependency = self
      .composes_order
      .source_order()
      .into_iter()
      .collect::<FxHashMap<_, _>>();

    for dep in &mut self.dependencies {
      let dependency_id = *dep.id();
      let Some(source_order) = source_order_by_dependency.get(&dependency_id) else {
        continue;
      };
      if let Some(dep) = dep.downcast_mut::<CssComposeDependency>() {
        dep.set_source_order(*source_order);
      }
    }
  }

  fn handle_icss_export_value(&mut self, prop: &str, value: &str) {
    let convention = self.convention();
    let convention_names = export_locals_convention(prop, convention);
    let value = REGEX_IS_COMMENTS.replace_all(value, "");
    let definition = self.resolve_icss_definition(value.as_ref());
    self
      .icss_definitions
      .insert(prop.to_string(), definition.clone());
    for name in convention_names.iter() {
      self.update_css_exports_from_icss_definition(name, prop, &definition);
    }
    if let Some(custom_property_name) = prop.strip_prefix("--") {
      self
        .icss_definitions
        .insert(custom_property_name.to_string(), definition.clone());
      for name in export_locals_convention(custom_property_name, convention).iter() {
        self.update_css_exports_from_custom_property_definition(name, prop, &definition);
      }
    }
    self
      .dependencies
      .push(Box::new(CssExportDependency::new(convention_names)));
  }

  fn handle_icss_import_from(&mut self, path: &str) {
    let path = self.resolve_icss_import_request(path);
    self.current_icss_import_from = Some(path);
  }

  fn handle_icss_import_value(&mut self, prop: &str, value: &str) {
    let Some(request) = self.current_icss_import_from.clone() else {
      return;
    };
    let definition = IcssDefinition::Import {
      import_name: value.to_string(),
      request: request.clone(),
    };
    self
      .icss_definitions
      .insert(prop.to_string(), definition.clone());
    if let Some(custom_property_name) = prop.strip_prefix("--") {
      self
        .icss_definitions
        .insert(custom_property_name.to_string(), definition.clone());
      for name in export_locals_convention(custom_property_name, self.convention()).iter() {
        self.update_css_exports_from_custom_property_definition(name, prop, &definition);
      }
    }
    self.dependencies.push(Box::new(CssComposeDependency::new(
      request,
      vec![value.to_owned().into()],
      DependencyRange::new(0, 0),
    )));
  }

  fn handle_icss_symbol(&mut self, name: &str, range: css_module_lexer::Range) {
    let Some(definition) = self.icss_definitions.get(name).cloned() else {
      return;
    };
    self
      .dependencies
      .push(Box::new(CssIcssSymbolDependency::new(
        self.icss_symbol_value_from_definition(name, &definition),
        (range.start, range.end).into(),
      )));
  }

  fn resolve_icss_definition(&self, value: &str) -> IcssDefinition {
    self
      .icss_definitions
      .get(value)
      .cloned()
      .unwrap_or_else(|| IcssDefinition::Value(value.to_string()))
  }

  fn resolve_icss_import_request(&self, path: &str) -> String {
    let path = path.trim_matches(|c| c == '\'' || c == '"');
    if let Some(IcssDefinition::Value(value)) = self.icss_definitions.get(path) {
      value.trim_matches(|c| c == '\'' || c == '"').to_string()
    } else if !path.starts_with('.')
      && !path.starts_with('/')
      && let Some(resource_path) = self.resource_data().path()
      && let Some(parent) = Path::new(resource_path.as_str()).parent()
      && parent.join(path).exists()
    {
      format!("./{path}")
    } else {
      path.to_string()
    }
  }

  fn update_css_exports_from_icss_definition(
    &mut self,
    name: &str,
    prop: &str,
    definition: &IcssDefinition,
  ) {
    let (ident, from) = match definition {
      IcssDefinition::Value(value) => (value.as_str(), None),
      IcssDefinition::Import {
        import_name,
        request,
      } => (import_name.as_str(), Some(request.as_str())),
    };
    update_css_exports(
      &mut self.css_exports,
      name,
      CssExport {
        ident: ident.into(),
        from: from.map(Into::into),
        id: None,
        orig_name: prop.into(),
      },
    );
  }

  fn update_css_exports_from_custom_property_definition(
    &mut self,
    name: &str,
    prop: &str,
    definition: &IcssDefinition,
  ) {
    if let IcssDefinition::Value(value) = definition
      && let Some(custom_property_name) = value.trim().strip_prefix("--")
      && is_custom_property_name(custom_property_name)
      && let Some(exports) = self.css_exports.get(custom_property_name).cloned()
    {
      for export in exports.iter() {
        update_css_exports(
          &mut self.css_exports,
          name,
          CssExport {
            ident: export.ident.clone(),
            from: export.from.clone(),
            id: export.id,
            orig_name: prop.into(),
          },
        );
      }
      return;
    }

    self.update_css_exports_from_icss_definition(name, prop, definition);
  }

  fn icss_symbol_value_from_definition(
    &self,
    name: &str,
    definition: &IcssDefinition,
  ) -> CssIcssSymbolValue {
    match definition {
      IcssDefinition::Value(value) => CssIcssSymbolValue::Literal(value.clone()),
      IcssDefinition::Import {
        import_name,
        request,
      } => CssIcssSymbolValue::Import {
        local_name: name.to_string(),
        import_name: import_name.clone(),
        request: request.clone(),
      },
    }
  }

  fn get_icss_composes_exports(&self, name: &str) -> Option<Vec<CssExport>> {
    let definition = self.icss_definitions.get(name)?;
    match definition {
      IcssDefinition::Value(value) => self
        .css_exports
        .get(value.as_str())
        .map(|exports| exports.iter().cloned().collect())
        .or_else(|| {
          Some(vec![CssExport {
            ident: value.as_str().into(),
            orig_name: name.into(),
            from: None,
            id: None,
          }])
        }),
      IcssDefinition::Import {
        import_name,
        request,
      } => Some(vec![CssExport {
        ident: import_name.as_str().into(),
        orig_name: name.into(),
        from: Some(request.as_str().into()),
        id: None,
      }]),
    }
  }

  fn add_warnings(&mut self, warnings: Vec<css_module_lexer::Warning>) {
    for warning in warnings {
      let range = warning.range();
      let error = css_parsing_traceable_error(
        &self.source_code,
        range.start,
        range.end,
        warning.to_string(),
        if matches!(
          warning.kind(),
          css_module_lexer::WarningKind::NotPrecededAtImport
            | css_module_lexer::WarningKind::NotPure { .. }
        ) {
          Severity::Error
        } else {
          Severity::Warning
        },
      );
      self.diagnostics.push(error.into());
    }
  }

  fn resource_data(&self) -> &'context ResourceData {
    self
      .parse_context
      .module_match_resource
      .unwrap_or(self.parse_context.resource_data)
  }
}

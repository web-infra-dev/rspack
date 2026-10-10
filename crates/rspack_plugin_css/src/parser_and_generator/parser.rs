use std::{path::Path, sync::Arc};

use rspack_core::{
  BoxDependency, ConstDependency, CssAutoOrModuleParserOptions, CssExportType, CssExports,
  CssExportsConvention, CssLayer, CssLocalNames, CssModuleGeneratorOptions,
  CssModuleRenderCondition, CssParserImport, CssParserImportContext, Dependency,
  DependencyCodeGenerationRef, DependencyId, DependencyRange, ModuleType, ParseContext,
  ParseResult, ResourceData, StaticExportsDependency, StaticExportsSpec,
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics, remove_bom, rspack_sources::Source,
  topological_sort,
};
use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, Severity, TWithDiagnosticArray};
use rspack_plugin_javascript::{RawMagicComment, try_extract_magic_comment_from_comments};
use rspack_util::fx_hash::FxIndexMap;
use rustc_hash::{FxHashMap, FxHashSet};
use smol_str::SmolStr;

use super::is_css_module;
use crate::{
  css_syntax::{normalize_url, unescape_identifier},
  dependency::{
    CssIcssExportDependency, CssIcssImportDependency, CssIcssReference, CssIcssSymbolDependency,
    CssIcssSymbolKind, CssImportDependency, CssLocalIdentKind, CssUrlDependency,
  },
  utils::{
    css_parsing_traceable_error, export_locals_convention, replace_module_request_prefix,
    source_order_to_i32,
  },
};

pub(super) struct CssModuleParser<'context> {
  parser_options: &'context CssAutoOrModuleParserOptions,
  generator_options: &'context CssModuleGeneratorOptions,
  export_type: Option<CssExportType>,
  has_charset: bool,
  parse_context: ParseContext<'context>,
  source: Arc<dyn Source>,
  source_code: Arc<str>,
  diagnostics: Vec<Diagnostic>,
  // Owns dependencies in parse order and allows definition lookup by ID.
  dependencies: FxIndexMap<DependencyId, BoxDependency>,
  presentational_dependencies: Vec<DependencyCodeGenerationRef>,
  code_generation_dependencies: Vec<DependencyId>,
  // Public JS export names after exportsConvention; colliding aliases may
  // point to a combined definition containing several local identifiers.
  css_exports: CssExports,
  // Raw local names encountered in declarations or references. Composes uses
  // these bindings to resolve a class independently of public export aliases.
  local_ident_definitions: FxHashMap<SmolStr, DependencyId>,
  // Raw dashed local names (such as custom properties), kept separate from
  // ordinary identifiers because they require a different generated name.
  local_dashed_ident_definitions: FxHashMap<SmolStr, DependencyId>,
  // Declared locals only, persisted in CssBuildInfo so codegen can report
  // unused generated identifiers to the CSS minimizer.
  css_local_names: CssLocalNames,
  // Unresolved local compose names mapped to placeholder IDs. Resolve them
  // after parsing to support declarations that appear later in the module.
  pending_local_composes: FxHashMap<SmolStr, DependencyId>,
  // Reuses literal definitions for names composed with `from global`.
  global_composes: FxHashMap<SmolStr, DependencyId>,
  // Current @value/:import/:export bindings, including whether a name can
  // interpolate individual identifiers or only replace a whole value.
  icss_definitions: FxHashMap<SmolStr, IcssBinding>,
  // Resolved request of the :import block whose bindings are being parsed.
  current_icss_import_from: Option<String>,
  // Tracks imported CSS order for the cascade before chunk graph construction.
  composes_order: ComposesOrderState,
}

#[derive(Clone, Copy)]
enum IcssInterpolation {
  WholeValue,
  Identifier,
}

struct IcssBinding {
  dependency: DependencyId,
  interpolation: IcssInterpolation,
}

/// Orders imported CSS modules for the cascade before chunk graph construction.
/// Export value expansion is handled separately by `CssModuleGenerator`.
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
}

fn is_custom_property_name(value: &str) -> bool {
  !value.is_empty()
    && value
      .bytes()
      .all(|c| !c.is_ascii_whitespace() && !matches!(c, b')' | b'(' | b'"' | b'\''))
}

fn normalize_ident_name(name: &str) -> SmolStr {
  SmolStr::new(unescape_identifier(name).as_ref())
}

fn raw_magic_comments<'a>(source: &str, comments: &'a str) -> Vec<RawMagicComment<'a>> {
  let source_start = source.as_ptr() as usize;
  let Some(base) = (comments.as_ptr() as usize)
    .checked_sub(source_start)
    .and_then(|base| base.checked_add(comments.len()).map(|end| (base, end)))
    .filter(|(base, end)| source.get(*base..*end) == Some(comments))
    .and_then(|(base, _)| u32::try_from(base).ok())
  else {
    return Vec::new();
  };
  let value = comments;
  let mut offset = 0;
  let mut result = Vec::new();

  while let Some(relative_start) = value[offset..].find("/*") {
    let start_offset = offset + relative_start;
    let content_start = start_offset + 2;
    let Some(relative_end) = value[content_start..].find("*/") else {
      break;
    };
    let content_end = content_start + relative_end;
    let end_offset = content_end + 2;
    let (Ok(start), Ok(end)) = (u32::try_from(start_offset), u32::try_from(end_offset)) else {
      break;
    };
    result.push(RawMagicComment {
      text: &value[content_start..content_end],
      span: DependencyRange::new(base + start, base + end),
    });
    offset = end_offset;
  }

  result
}

impl<'context> CssModuleParser<'context> {
  pub fn new(
    generator_options: &'context CssModuleGeneratorOptions,
    parser_options: &'context CssAutoOrModuleParserOptions,
    parse_context: ParseContext<'context>,
  ) -> Self {
    let source = remove_bom(parse_context.source.clone());
    let source_code: Arc<str> = source.source().into_string_lossy().into();

    let export_type = parse_context
      .build_info
      .css
      .as_deref()
      .and_then(|css_build_info| css_build_info.export_type)
      .or(parser_options.export_type);

    Self {
      parser_options,
      generator_options,
      export_type,
      has_charset: false,
      parse_context,
      source,
      source_code,
      diagnostics: vec![],
      dependencies: Default::default(),
      presentational_dependencies: vec![],
      code_generation_dependencies: vec![],
      css_exports: Default::default(),
      local_ident_definitions: Default::default(),
      local_dashed_ident_definitions: Default::default(),
      css_local_names: Default::default(),
      pending_local_composes: Default::default(),
      global_composes: Default::default(),
      icss_definitions: Default::default(),
      current_icss_import_from: None,
      composes_order: Default::default(),
    }
  }

  pub async fn parse(mut self) -> Result<TWithDiagnosticArray<ParseResult>> {
    let mode = self.mode();
    let deps_source_code = self.source_code.clone();
    let (deps, warnings) = css_module_lexer::collect_dependencies(&deps_source_code, mode);
    let local_css_ident_declarations =
      self.collect_local_css_ident_declarations(deps.dependencies());

    for dependency in &deps {
      self
        .handle_dependency(dependency, &deps, &local_css_ident_declarations)
        .await?;
    }

    self.resolve_pending_local_composes();
    self.apply_composes_source_order();
    self.add_warnings(warnings);

    if matches!(
      self.export_type(),
      Some(CssExportType::Text | CssExportType::CssStyleSheet)
    ) {
      self.add_dependency(StaticExportsDependency::new(
        StaticExportsSpec::Array(vec!["default".into()]),
        false,
      ));
    }

    let css_build_info = self.parse_context.build_info.css.get_or_insert_default();
    css_build_info.exports = self.css_exports;
    css_build_info.local_names = self.css_local_names;
    css_build_info.has_charset = self.has_charset;

    Ok(
      ParseResult {
        dependencies: self.dependencies.into_values().collect(),
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
        if is_css_module(
          self.parse_context.module_type,
          resource_path.map(|path| path.as_str()),
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

  fn export_type(&self) -> Option<CssExportType> {
    self.export_type
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
          declarations.vars.insert(normalize_ident_name(name));
        }
        css_module_lexer::Dependency::ICSSExportValue { prop, .. }
        | css_module_lexer::Dependency::ICSSImportValue { prop, .. }
          if self.dashed_idents()
            && prop.strip_prefix("--").is_some_and(is_custom_property_name) =>
        {
          let name = prop
            .strip_prefix("--")
            .expect("custom property was checked above");
          declarations.vars.insert(normalize_ident_name(name));
        }
        _ => {}
      }
    }

    declarations
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
    let line_end = bytes[end..]
      .iter()
      .position(|byte| *byte == b'\r' || *byte == b'\n')
      .map_or(bytes.len(), |pos| end + pos);
    let replacement_removes_line = bytes[end..line_end]
      .iter()
      .all(|byte| *byte == b' ' || *byte == b'\t');

    if replacement_removes_line
      && bytes[line_start..start]
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

  fn add_invalid_bare_import_warning(&mut self, range: &css_module_lexer::Range) {
    let when = self
      .source_code
      .get(range.start as usize..range.end as usize)
      .map_or("@import", str::trim);
    let error = css_parsing_traceable_error(
      &self.source_code,
      range.start,
      range.end,
      format!("Expected URL in '{when}'"),
      Severity::Warning,
    );
    self.diagnostics.push(error.into());
  }

  async fn handle_dependency<'source>(
    &mut self,
    dependency: &css_module_lexer::Dependency<'source>,
    dependency_context: &css_module_lexer::DependencyContext<'source>,
    local_css_ident_declarations: &LocalCssIdentDeclarations,
  ) -> Result<()> {
    match dependency {
      css_module_lexer::Dependency::Url {
        request,
        range,
        kind,
        magic_comments,
      } => {
        if self.url() && self.should_ignore_magic_comments(*magic_comments, *range) {
          return Ok(());
        }
        self.handle_url(request, *range, *kind)
      }
      css_module_lexer::Dependency::Import {
        request,
        range,
        attributes,
        magic_comments,
      } => {
        if self.import() && self.should_ignore_magic_comments(*magic_comments, *range) {
          return Ok(());
        }
        let attributes = dependency_context.import_attributes(*attributes);
        self
          .handle_import(
            request,
            *range,
            attributes.media(),
            attributes.supports(),
            attributes.layer(),
          )
          .await
      }
      css_module_lexer::Dependency::Replace { content, range } => {
        let range = self.presentational_replace_range(content, *range);
        self
          .presentational_dependencies
          .push(Arc::new(ConstDependency::new(range, (*content).into())));
        Ok(())
      }
      css_module_lexer::Dependency::Charset { range, .. } => {
        self.handle_charset(*range);
        Ok(())
      }
      css_module_lexer::Dependency::LocalClass { name, range, .. }
      | css_module_lexer::Dependency::LocalId { name, range, .. } => {
        self.reset_current_composes_rule();
        let (_prefix, name) = name.split_at(1);
        self.handle_local_ident_declaration(name, range.start + 1, range.end)
      }
      css_module_lexer::Dependency::LocalKeyframes { name, range, .. } => self
        .handle_optional_local_ident_usage(
          self.animation() && local_css_ident_declarations.has_keyframes(name),
          name,
          *range,
        ),
      css_module_lexer::Dependency::LocalKeyframesDecl { name, range, .. } => {
        self.handle_optional_local_ident_declaration(self.animation(), name, *range)
      }
      css_module_lexer::Dependency::Composes {
        local_classes,
        names,
        from,
        from_is_global,
        range,
      } => {
        self.handle_composes(
          dependency_context
            .composes_local_classes(*local_classes)
            .iter()
            .copied(),
          dependency_context.composes_names(*names).iter().copied(),
          *from,
          *from_is_global,
          *range,
        );
        Ok(())
      }
      css_module_lexer::Dependency::ICSSExportValue {
        prop,
        value,
        is_at_value,
        identifiers,
      } => {
        self.handle_icss_export_value(
          prop,
          value.as_ref(),
          *is_at_value,
          dependency_context.value_at_rule_identifiers(*identifiers),
        );
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
      css_module_lexer::Dependency::ICSSImportUrl { name, range, .. } => {
        if let Some(request) = self.resolve_icss_import_url_request(name) {
          self.handle_import(&request, *range, None, None, None).await
        } else {
          self.add_invalid_bare_import_warning(range);
          Ok(())
        }
      }
      css_module_lexer::Dependency::ICSSSymbol { name, range } => {
        self.handle_icss_symbol(name, *range);
        Ok(())
      }
      css_module_lexer::Dependency::LocalCounterStyle { name, range, .. }
      | css_module_lexer::Dependency::LocalFontPalette { name, range, .. } => self
        .handle_optional_local_ident_usage(
          self.custom_idents() && local_css_ident_declarations.has_custom_ident(name),
          name,
          *range,
        ),
      css_module_lexer::Dependency::LocalCounterStyleDecl { name, range, .. }
      | css_module_lexer::Dependency::LocalFontPaletteDecl { name, range, .. } => {
        self.handle_optional_local_ident_declaration(self.custom_idents(), name, *range)
      }
      css_module_lexer::Dependency::LocalContainer { name, range, .. } => self
        .handle_optional_local_ident_usage(
          self.container() && local_css_ident_declarations.has_container(name),
          name,
          *range,
        ),
      css_module_lexer::Dependency::LocalContainerDecl { name, range, .. } => {
        self.handle_optional_local_ident_declaration(self.container(), name, *range)
      }
      css_module_lexer::Dependency::LocalFunction { name, range, .. } => self
        .handle_optional_local_dashed_ident_usage(
          self.function() && local_css_ident_declarations.has_function(name),
          name,
          *range,
        ),
      css_module_lexer::Dependency::LocalFunctionDecl { name, range, .. } => {
        self.handle_optional_local_dashed_ident_declaration(self.function(), name, *range)
      }
      css_module_lexer::Dependency::LocalGrid { name, range, .. } => self
        .handle_optional_local_ident_usage(
          self.grid() && local_css_ident_declarations.has_grid(name),
          name,
          *range,
        ),
      css_module_lexer::Dependency::LocalGridDecl { name, range, .. } => {
        self.handle_optional_local_ident_declaration(self.grid(), name, *range)
      }
      css_module_lexer::Dependency::LocalVar {
        name,
        range,
        from,
        from_is_global,
      } => {
        if !self.dashed_idents() {
          return Ok(());
        }
        self.handle_local_var_usage(
          name,
          *range,
          *from,
          *from_is_global,
          local_css_ident_declarations,
        )
      }
      css_module_lexer::Dependency::LocalVarDecl { name, range, .. }
      | css_module_lexer::Dependency::LocalPropertyDecl { name, range, .. } => {
        self.handle_optional_local_var_declaration(self.dashed_idents(), name, *range)
      }
    }
  }

  fn should_ignore_magic_comments(
    &mut self,
    comments: Option<&str>,
    range: css_module_lexer::Range,
  ) -> bool {
    let Some(comments) = comments else {
      return false;
    };
    let comments = raw_magic_comments(&self.source_code, comments);
    let (options, diagnostics) = try_extract_magic_comment_from_comments(
      &self.source_code,
      &comments,
      DependencyRange::new(range.start, range.end),
    );
    self.diagnostics.extend(diagnostics);
    options.get_ignore() == Some(true)
  }

  fn handle_charset(&mut self, range: css_module_lexer::Range) {
    self.has_charset = true;
    self
      .presentational_dependencies
      .push(Arc::new(ConstDependency::new(
        (range.start, range.end).into(),
        "".into(),
      )));
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
    // Fragment-only URLs reference the current document, not another module.
    if request.starts_with('#') {
      return Ok(());
    }
    let dep = CssUrlDependency::new(
      request.into_owned(),
      DependencyRange::new(range.start, range.end),
      matches!(kind, css_module_lexer::UrlRangeKind::Function),
    );
    self.code_generation_dependencies.push(*dep.id());
    self.add_dependency(dep);
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
        .push(Arc::new(ConstDependency::new(
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
    let dep = CssImportDependency::new(
      request,
      DependencyRange::new(range.start, range.end),
      inherited_render_conditions,
      render_condition,
      self.export_type(),
    );
    self.code_generation_dependencies.push(*dep.id());
    self.add_dependency(dep);
    Ok(())
  }

  fn css_import_inherited_render_conditions(&self) -> Vec<CssModuleRenderCondition> {
    let (mut inherited_render_conditions, render_condition) = self
      .parse_context
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

    if render_condition.is_empty() {
      return inherited_render_conditions;
    }
    inherited_render_conditions.push(render_condition);
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

  fn handle_local_ident_usage(&mut self, name: &str, range: css_module_lexer::Range) -> Result<()> {
    let name = unescape_identifier(name);
    self.add_local_symbol(name.as_ref(), range, CssLocalIdentKind::Ident)
  }

  fn handle_optional_local_ident_usage(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    self.handle_local_ident_usage(name, range)
  }

  fn handle_local_ident_declaration(&mut self, name: &str, start: u32, end: u32) -> Result<()> {
    let name = unescape_identifier(name);
    self.add_local_declaration(name.as_ref(), (start, end).into(), CssLocalIdentKind::Ident)
  }

  fn handle_local_var_usage(
    &mut self,
    name: &str,
    range: css_module_lexer::Range,
    from: Option<&str>,
    from_is_global: bool,
    local_css_ident_declarations: &LocalCssIdentDeclarations,
  ) -> Result<()> {
    let name = unescape_identifier(name);

    if from_is_global {
      return Ok(());
    }

    if from.is_none() && !local_css_ident_declarations.vars.contains(name.as_ref()) {
      return Ok(());
    }

    self.add_local_symbol(name.as_ref(), range, CssLocalIdentKind::DashedIdent)?;
    Ok(())
  }

  // `name` is decoded by the syntax-specific caller exactly once.
  fn add_local_symbol(
    &mut self,
    name: &str,
    range: css_module_lexer::Range,
    kind: CssLocalIdentKind,
  ) -> Result<()> {
    let id = self.ensure_local_definition(name, kind, false);
    self.add_dependency(CssIcssSymbolDependency::new(
      id,
      (range.start, range.end).into(),
      CssIcssSymbolKind::LocalReference,
    ));
    Ok(())
  }

  fn handle_optional_local_dashed_ident_usage(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    let name = unescape_identifier(name);
    self.add_local_symbol(name.as_ref(), range, CssLocalIdentKind::DashedIdent)
  }

  fn handle_local_var_declaration(&mut self, name: &str, start: u32, end: u32) -> Result<()> {
    let name = unescape_identifier(name);
    self.add_local_declaration(
      name.as_ref(),
      (start, end).into(),
      CssLocalIdentKind::DashedIdent,
    )
  }

  fn add_local_declaration(
    &mut self,
    name: &str,
    range: DependencyRange,
    kind: CssLocalIdentKind,
  ) -> Result<()> {
    let id = self.ensure_local_definition(name, kind, true);
    self.add_dependency(CssIcssSymbolDependency::new(
      id,
      range,
      CssIcssSymbolKind::LocalDeclaration,
    ));
    Ok(())
  }

  fn handle_optional_local_var_declaration(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    self.handle_local_var_declaration(name, range.start, range.end)
  }

  fn handle_optional_local_dashed_ident_declaration(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    self.handle_local_var_declaration(name, range.start, range.end)
  }

  fn handle_optional_local_ident_declaration(
    &mut self,
    enabled: bool,
    name: &str,
    range: css_module_lexer::Range,
  ) -> Result<()> {
    if !enabled {
      return Ok(());
    }
    self.handle_local_ident_declaration(name, range.start, range.end)
  }

  fn add_dependency(&mut self, dependency: impl Dependency + 'static) -> DependencyId {
    let id = *dependency.id();
    self.dependencies.insert(id, BoxDependency::new(dependency));
    id
  }

  fn definition(&self, id: DependencyId) -> Option<&CssIcssExportDependency> {
    self.dependencies.get(&id)?.downcast_ref()
  }

  fn definition_mut(&mut self, id: DependencyId) -> &mut CssIcssExportDependency {
    self.dependencies[&id]
      .downcast_mut()
      .expect("CSS export definition should exist")
  }

  fn index_definition(&mut self, name: &str, id: DependencyId) {
    for alias in export_locals_convention(name, self.convention()) {
      self.css_exports.insert(alias.into(), id);
    }
  }

  fn local_ident_definition(&self, name: &str) -> Option<DependencyId> {
    // CSS compositions use raw local names; convention aliases are JS exports.
    self.local_ident_definitions.get(name).copied()
  }

  fn contains_local_definition(&self, mut export: DependencyId, local: DependencyId) -> bool {
    loop {
      if export == local {
        return true;
      }
      let Some(definition) = self.definition(export) else {
        return false;
      };
      if definition.composes.contains(&local) {
        return true;
      }
      let [reference] = definition.references.as_slice() else {
        return false;
      };
      if !definition.value.is_empty() || reference.range.start != 0 || reference.range.end != 0 {
        return false;
      }
      // Alias collisions wrap an earlier export in an empty definition, with
      // newly appended locals in `composes`. Follow that chain to find all
      // existing members without expanding value substitutions or compositions.
      export = reference.dependency_id;
    }
  }

  fn ensure_local_definition(
    &mut self,
    name: &str,
    kind: CssLocalIdentKind,
    declared: bool,
  ) -> DependencyId {
    let existing = match kind {
      CssLocalIdentKind::Ident => self.local_ident_definitions.get(name),
      CssLocalIdentKind::DashedIdent => self.local_dashed_ident_definitions.get(name),
    };
    let id = if let Some(id) = existing {
      *id
    } else {
      let id = self.add_dependency(CssIcssExportDependency::new(
        name.into(),
        name.into(),
        vec![],
        Some(kind),
        None,
      ));
      match kind {
        CssLocalIdentKind::Ident => self.local_ident_definitions.insert(name.into(), id),
        CssLocalIdentKind::DashedIdent => {
          self.local_dashed_ident_definitions.insert(name.into(), id)
        }
      };
      id
    };
    if declared {
      self.definition_mut(id).can_mangle = Some(true);
      self.css_local_names.insert(name.into(), id);
    }
    let mut appended = FxHashMap::default();
    for alias in export_locals_convention(name, self.convention()) {
      let custom_export = self
        .css_exports
        .get(alias.as_str())
        .and_then(|id| self.definition(*id))
        .is_some_and(|dep| {
          dep.local_ident == Some(CssLocalIdentKind::DashedIdent)
            || (dep.local_ident.is_none()
              && (dep.value.starts_with("--")
                || dep
                  .value
                  .strip_prefix("_--")
                  .is_some_and(is_custom_property_name)))
        });
      if kind == CssLocalIdentKind::Ident && custom_export {
        continue;
      }
      let export = if let Some(previous) = self.css_exports.get(alias.as_str()).copied() {
        if self.contains_local_definition(previous, id) {
          continue;
        }
        let previous_definition = self
          .definition(previous)
          .expect("CSS export definition should exist");
        if let Some(export) = appended.get(&previous) {
          *export
        } else {
          // Appending a class must not mutate an earlier @value definition:
          // its existing symbol uses still refer to the original value alone.
          let mut export = CssIcssExportDependency::new(
            name.into(),
            "".into(),
            vec![CssIcssReference {
              range: (0, 0).into(),
              dependency_id: previous,
            }],
            None,
            Some(previous_definition.can_mangle.unwrap_or(true)),
          );
          export.composes.push(id);
          let export = self.add_dependency(export);
          appended.insert(previous, export);
          export
        }
      } else {
        id
      };
      self.css_exports.insert(alias.into(), export);
    }
    id
  }

  fn handle_composes<'source>(
    &mut self,
    local_classes: impl IntoIterator<Item = &'source str>,
    names: impl IntoIterator<Item = &'source str>,
    from: Option<&'source str>,
    from_is_global: bool,
    range: css_module_lexer::Range,
  ) {
    let local_classes = local_classes
      .into_iter()
      .map(|name| unescape_identifier(name).into_owned())
      .collect::<Vec<_>>();
    let request = if from_is_global {
      None
    } else {
      from.map(|from| self.resolve_icss_import_request(from))
    };
    let mut seen_names = FxHashSet::default();
    for name in names {
      let name = unescape_identifier(name);
      if !seen_names.insert(SmolStr::new(&name)) {
        continue;
      }
      let target = if let Some(request) = &request {
        let dep = CssIcssImportDependency::new(
          request.clone(),
          name.as_ref().into(),
          name.as_ref().into(),
          (range.start, range.end).into(),
          self.export_type(),
        );
        let id = *dep.id();
        self
          .composes_order
          .track_request_order(&local_classes, request, range.start, id);
        self.add_dependency(dep);
        id
      } else if from_is_global {
        if let Some(id) = self.global_composes.get(name.as_ref()) {
          *id
        } else {
          let id = self.add_dependency(CssIcssExportDependency::new(
            name.as_ref().into(),
            name.as_ref().into(),
            vec![],
            None,
            None,
          ));
          self.global_composes.insert(name.as_ref().into(), id);
          id
        }
      } else if let Some(binding) = self.icss_definitions.get(name.as_ref()) {
        // A local @value can name a local class; follow that definition without
        // copying its value or building a synthetic module request.
        self
          .literal_definition(binding.dependency)
          .and_then(|value| self.local_ident_definition(value))
          .unwrap_or(binding.dependency)
      } else if let Some(id) = self.local_ident_definition(name.as_ref()) {
        id
      } else {
        *self
          .pending_local_composes
          .entry(name.as_ref().into())
          .or_default()
      };
      for local in &local_classes {
        if let Some(id) = self.local_ident_definition(local) {
          let dep = self.definition_mut(id);
          if !dep.composes.contains(&target) {
            dep.composes.push(target);
          }
        }
      }
    }
  }

  fn resolve_pending_local_composes(&mut self) {
    if self.pending_local_composes.is_empty() {
      return;
    }
    // Resolve against the completed module, including later declarations.
    // Keep each composition's original position when replacing its placeholder.
    let mut local_targets = FxHashMap::default();
    for (name, placeholder) in std::mem::take(&mut self.pending_local_composes) {
      let target = if let Some(id) = self.local_ident_definition(&name) {
        id
      } else {
        self.add_dependency(CssIcssExportDependency::new(
          name.clone(),
          name,
          vec![],
          None,
          None,
        ))
      };
      local_targets.insert(placeholder, target);
    }
    for dependency in self.dependencies.values_mut() {
      let Some(definition) = dependency.downcast_mut::<CssIcssExportDependency>() else {
        continue;
      };
      for target in &mut definition.composes {
        if let Some(id) = local_targets.get(target) {
          *target = *id;
        }
      }
      if definition.composes.len() > 1 {
        let mut seen = FxHashSet::default();
        definition.composes.retain(|id| seen.insert(*id));
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

    for dep in self.dependencies.values_mut() {
      let dependency_id = *dep.id();
      let Some(source_order) = source_order_by_dependency.get(&dependency_id) else {
        continue;
      };
      if let Some(dep) = dep.downcast_mut::<CssIcssImportDependency>() {
        dep.set_source_order(*source_order);
      }
    }
  }

  fn handle_icss_export_value(
    &mut self,
    prop: &str,
    value: &str,
    is_at_value: bool,
    identifiers: &[css_module_lexer::Range],
  ) {
    let references = self.icss_references(value, identifiers);
    let id = self.add_dependency(CssIcssExportDependency::new(
      prop.into(),
      value.into(),
      references,
      None,
      Some(false),
    ));
    self.bind_icss(
      prop,
      id,
      if is_at_value {
        IcssInterpolation::Identifier
      } else {
        IcssInterpolation::WholeValue
      },
    );
    self.index_definition(prop, id);
  }

  fn bind_icss(&mut self, name: &str, dependency: DependencyId, interpolation: IcssInterpolation) {
    self
      .icss_definitions
      .entry(name.into())
      .and_modify(|binding| {
        binding.dependency = dependency;
        // Once introduced by @value or :import, the name remains eligible for
        // interpolation even when a later :export replaces its value.
        if matches!(interpolation, IcssInterpolation::Identifier) {
          binding.interpolation = interpolation;
        }
      })
      .or_insert(IcssBinding {
        dependency,
        interpolation,
      });
  }

  fn handle_icss_import_from(&mut self, path: &str) {
    self.current_icss_import_from = Some(self.resolve_icss_import_request(path));
  }

  fn handle_icss_import_value(&mut self, prop: &str, value: &str) {
    let Some(request) = self.current_icss_import_from.clone() else {
      return;
    };
    let dep = CssIcssImportDependency::new(
      request,
      value.into(),
      prop.into(),
      (0, 0).into(),
      self.export_type(),
    );
    let id = *dep.id();
    self.add_dependency(dep);
    self.bind_icss(prop, id, IcssInterpolation::Identifier);
    if let Some(name) = prop.strip_prefix("--") {
      self.bind_icss(name, id, IcssInterpolation::WholeValue);
      let export = self.add_dependency(CssIcssExportDependency::new(
        name.into(),
        "".into(),
        vec![CssIcssReference {
          range: (0, 0).into(),
          dependency_id: id,
        }],
        None,
        None,
      ));
      self.index_definition(name, export);
    }
  }

  fn handle_icss_symbol(&mut self, name: &str, range: css_module_lexer::Range) {
    let Some(binding) = self.icss_definitions.get(name) else {
      return;
    };
    self.add_dependency(CssIcssSymbolDependency::new(
      binding.dependency,
      (range.start, range.end).into(),
      CssIcssSymbolKind::IcssReference,
    ));
  }

  fn icss_references(
    &self,
    value: &str,
    identifiers: &[css_module_lexer::Range],
  ) -> Vec<CssIcssReference> {
    if let Some(binding) = self.icss_definitions.get(value) {
      return vec![CssIcssReference {
        range: (0, value.len() as u32).into(),
        dependency_id: binding.dependency,
      }];
    }
    // Reuse the lexer ranges. Functions, quoted strings and comments are not
    // symbol references, and definitions keep the surrounding text verbatim.
    identifiers
      .iter()
      .filter_map(|range| {
        let name = &value[range.start as usize..range.end as usize];
        let binding = self.icss_definitions.get(name)?;
        if !matches!(binding.interpolation, IcssInterpolation::Identifier) {
          return None;
        }
        Some(CssIcssReference {
          range: (range.start, range.end).into(),
          dependency_id: binding.dependency,
        })
      })
      .collect()
  }

  fn literal_definition(&self, mut id: DependencyId) -> Option<&str> {
    let mut seen = FxHashSet::default();
    while seen.insert(id) {
      let dep = self.definition(id)?;
      if dep.references.is_empty() {
        return Some(&dep.value);
      }
      let reference = dep.references.first()?;
      if dep.references.len() != 1
        || reference.range.start != 0
        || reference.range.end as usize != dep.value.len()
      {
        return None;
      }
      id = reference.dependency_id;
    }
    None
  }

  fn resolve_icss_import_request(&self, path: &str) -> String {
    let path = path.trim().trim_matches(|c| c == '\'' || c == '"');
    if let Some(value) = self
      .icss_definitions
      .get(path)
      .and_then(|binding| self.literal_definition(binding.dependency))
    {
      value.trim_matches(|c| c == '\'' || c == '"').to_owned()
    } else if !path.starts_with('.')
      && !path.starts_with('/')
      && let Some(resource_path) = self.resource_data().path()
      && let Some(parent) = Path::new(resource_path.as_str()).parent()
      && parent.join(path).exists()
    {
      format!("./{path}")
    } else {
      path.to_owned()
    }
  }

  fn resolve_icss_import_url_request(&self, name: &str) -> Option<String> {
    let name = name.trim().trim_matches(|c| c == '\'' || c == '"');
    self.literal_definition(self.icss_definitions.get(name)?.dependency)?;
    let request = self.resolve_icss_import_request(name);
    (!request.trim().is_empty()).then_some(request)
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

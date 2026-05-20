use std::sync::Arc;

use once_cell::sync::OnceCell;
use rspack_core::{
  BoxDependencyTemplate, BoxModuleDependency, BuildMetaDefaultObject, BuildMetaExportsType,
  ConstDependency, CssExport, CssExports, CssExportsConvention, CssModuleGeneratorOptions,
  CssModuleParserOptions, CssParserImport, CssParserImportContext, Dependency, DependencyRange,
  ModuleType, ParseContext, ParseResult, ResourceData,
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics, remove_bom, rspack_sources::Source,
};
use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, Severity, TWithDiagnosticArray};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{REGEX_CUSTOM_PROPERTY_IDENT, REGEX_IS_COMMENTS, REGEX_IS_MODULES};
use crate::{
  dependency::{
    CssComposeDependency, CssExportDependency, CssImportDependency, CssLayer,
    CssLocalIdentDependency, CssSelfReferenceLocalIdentDependency,
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
  parser_options: &'context CssModuleParserOptions,
  exports_only: bool,
  parse_context: ParseContext<'context>,
  source_code: Arc<str>,
  diagnostics: Vec<Diagnostic>,
  dependencies: Vec<Box<dyn Dependency>>,
  presentational_dependencies: Vec<BoxDependencyTemplate>,
  code_generation_dependencies: Vec<BoxModuleDependency>,
  css_exports: CssExports,
  css_local_names: FxHashMap<String, String>,
  local_ident_options: OnceCell<LocalIdentOptions<'context>>,
}

impl<'context> CssModuleParser<'context> {
  pub fn new(
    generator_options: &'context CssModuleGeneratorOptions,
    parser_options: &'context CssModuleParserOptions,
    exports_only: bool,
    parse_context: ParseContext<'context>,
  ) -> Self {
    let source_code: Arc<str> = remove_bom(parse_context.source.clone())
      .source()
      .into_string_lossy()
      .into();

    Self {
      generator_options,
      parser_options,
      exports_only,
      parse_context,
      source_code,
      diagnostics: vec![],
      dependencies: vec![],
      presentational_dependencies: vec![],
      code_generation_dependencies: vec![],
      css_exports: Default::default(),
      css_local_names: Default::default(),
      local_ident_options: OnceCell::new(),
    }
  }

  pub async fn parse(mut self) -> Result<TWithDiagnosticArray<ParseResult>> {
    self.prepare_build_meta();

    let mode = self.mode();
    let deps_source_code = self.source_code.clone();
    let (deps, warnings) = css_module_lexer::collect_dependencies(&deps_source_code, mode);
    let module_hash_options = self.create_module_hash_options(&deps);

    for dependency in deps {
      self
        .handle_dependency(dependency, &module_hash_options)
        .await?;
    }

    self.add_warnings(warnings);
    self.parse_context.build_info.css_exports = if self.css_exports.is_empty() {
      None
    } else {
      Some(self.css_exports)
    };
    self.parse_context.build_info.css_local_names = if self.css_local_names.is_empty() {
      None
    } else {
      Some(self.css_local_names)
    };

    Ok(
      ParseResult {
        dependencies: self.dependencies,
        blocks: vec![],
        presentational_dependencies: self.presentational_dependencies,
        code_generation_dependencies: self.code_generation_dependencies,
        source: remove_bom(self.parse_context.source),
        side_effects_bailout: None,
      }
      .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
        self.diagnostics,
        self.parse_context.loaders,
      )),
    )
  }

  fn prepare_build_meta(&mut self) {
    self.parse_context.build_info.strict = true;
    self.parse_context.build_meta.exports_type = if self.named_exports() {
      BuildMetaExportsType::Namespace
    } else {
      BuildMetaExportsType::Default
    };
    self.parse_context.build_meta.default_object = if self.named_exports() {
      BuildMetaDefaultObject::False
    } else {
      BuildMetaDefaultObject::Redirect
    };
  }

  fn mode(&self) -> css_module_lexer::Mode {
    let resource_path = self.resource_data().path();
    match self.parse_context.module_type {
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
        css_module_lexer::Mode::Local
      }
      _ => css_module_lexer::Mode::Css,
    }
  }

  fn create_module_hash_options<'source>(
    &self,
    deps: &[css_module_lexer::Dependency<'source>],
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
        | css_module_lexer::Dependency::LocalKeyframesDecl { name, .. }
          if self.animation() =>
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
        css_module_lexer::Dependency::LocalCounterStyle { name, .. }
        | css_module_lexer::Dependency::LocalCounterStyleDecl { name, .. }
        | css_module_lexer::Dependency::LocalFontPalette { name, .. }
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
        css_module_lexer::Dependency::LocalVar { name, .. }
        | css_module_lexer::Dependency::LocalVarDecl { name, .. }
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

    if let Some(convention) = convention {
      for captures in REGEX_CUSTOM_PROPERTY_IDENT.captures_iter(&self.source_code) {
        if let Some(name) = captures.get(2) {
          self.collect_export_dependency_name(
            name.as_str().to_string(),
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

  async fn handle_dependency<'source>(
    &mut self,
    dependency: css_module_lexer::Dependency<'source>,
    module_hash_options: &LocalIdentModuleHashOptions<'_>,
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
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            (range.start, range.end).into(),
            content.into(),
          )));
        Ok(())
      }
      css_module_lexer::Dependency::LocalClass { name, range, .. }
      | css_module_lexer::Dependency::LocalId { name, range, .. } => {
        let (_prefix, name) = name.split_at(1);
        self
          .handle_local_ident_declaration(name, range.start + 1, range.end, module_hash_options)
          .await
      }
      css_module_lexer::Dependency::LocalKeyframes { name, range, .. } => {
        self
          .handle_optional_local_ident_usage(self.animation(), name, range, module_hash_options)
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
      css_module_lexer::Dependency::LocalCounterStyle { name, range, .. }
      | css_module_lexer::Dependency::LocalFontPalette { name, range, .. } => {
        self
          .handle_optional_local_ident_usage(self.custom_idents(), name, range, module_hash_options)
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
      css_module_lexer::Dependency::LocalVar { name, range, .. } => {
        self
          .handle_optional_local_ident_usage(self.dashed_idents(), name, range, module_hash_options)
          .await
      }
      css_module_lexer::Dependency::LocalVarDecl { name, range, .. }
      | css_module_lexer::Dependency::LocalPropertyDecl { name, range, .. } => {
        self
          .handle_optional_local_ident_declaration(
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
    if request.is_empty() {
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          (range.start, range.end).into(),
          "".into(),
        )));
      return Ok(());
    }

    if !self.import() || !self.should_import(request, media, supports, layer).await {
      return Ok(());
    }

    let request = replace_module_request_prefix(
      request,
      &mut self.diagnostics,
      &self.source_code,
      range.start,
      range.end,
    );
    self.dependencies.push(Box::new(CssImportDependency::new(
      request.to_string(),
      DependencyRange::new(range.start, range.end),
      media.map(|s| s.to_string()),
      supports.map(|s| s.to_string()),
      layer.map(|s| {
        if s.is_empty() {
          CssLayer::Anonymous
        } else {
          CssLayer::Named(s.to_string())
        }
      }),
    )));
    Ok(())
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

  fn custom_idents(&self) -> bool {
    self.parser_options.custom_idents.unwrap_or(false)
  }

  fn dashed_idents(&self) -> bool {
    self.parser_options.dashed_idents.unwrap_or(false)
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
      .insert(name.into_owned(), local_ident.clone());

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
      update_css_exports(
        &mut self.css_exports,
        convention_name.to_owned(),
        CssExport {
          ident: local_ident.clone(),
          orig_name: name.to_owned(),
          from: None,
          id: None,
        },
      );
    }
    Ok((local_ident, convention_names))
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
          if let Some(existing) = self.css_exports.get(name.as_str())
            && from.is_none()
          {
            let existing = existing.clone();
            self
              .css_exports
              .get_mut(local_class.as_str())
              .expect("composes local class must already added to exports")
              .extend(existing);
          } else {
            self
              .css_exports
              .get_mut(local_class.as_str())
              .expect("composes local class must already added to exports")
              .insert(CssExport {
                ident: convention_name.clone(),
                orig_name: name.clone(),
                from: from
                  .filter(|f| *f != "global")
                  .map(|f| f.trim_matches(|c| c == '\'' || c == '"').to_string()),
                id: dep_id,
              });
          }
        }
      }
    }
  }

  fn handle_icss_export_value(&mut self, prop: &str, value: &str) {
    let convention = self.convention();
    let convention_names = export_locals_convention(prop, convention);
    let value = REGEX_IS_COMMENTS.replace_all(value, "");
    for name in convention_names.iter() {
      update_css_exports(
        &mut self.css_exports,
        name.to_owned(),
        CssExport {
          ident: value.to_string(),
          from: None,
          id: None,
          orig_name: prop.to_string(),
        },
      );
    }
    self
      .dependencies
      .push(Box::new(CssExportDependency::new(convention_names)));
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

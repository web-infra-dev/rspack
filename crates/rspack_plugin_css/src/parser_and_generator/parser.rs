use std::{
  hash::Hasher,
  sync::{Arc, LazyLock},
};

use once_cell::sync::OnceCell;
use regex::Regex;
use rspack_core::{
  BoxDependencyTemplate, BoxLoader, BoxModuleDependency, BuildInfo, BuildMeta,
  BuildMetaDefaultObject, BuildMetaExportsType, CompilerOptions, ConstDependency,
  CssExportsConvention, CssModuleGeneratorOptions, CssModuleParserOptions, CssParserImport,
  CssParserImportContext, Dependency, DependencyRange, LocalIdentName, ModuleType, ParseContext,
  ParseResult, ResourceData,
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics,
  remove_bom,
  rspack_sources::{BoxSource, Source},
};
use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, Severity, TWithDiagnosticArray};
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash};
use rspack_util::{identifier::make_paths_relative, itoa};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  dependency::{
    CssComposeDependency, CssExportDependency, CssImportDependency, CssLayer,
    CssLocalIdentDependency, CssSelfReferenceLocalIdentDependency,
    CssSelfReferenceLocalIdentReplacement, CssUrlDependency,
  },
  parser_and_generator::{CssExport, CssExports, generator::update_css_exports},
  utils::{
    LEADING_DIGIT_REGEX, LocalIdentOptions, css_parsing_traceable_error, export_locals_convention,
    normalize_url, replace_module_request_prefix, unescape,
  },
};

static REGEX_IS_MODULES: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\.module(s)?\.[^.]+$").expect("Invalid regex"));

static REGEX_IS_COMMENTS: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"/\*[\s\S]*?\*/").expect("Invalid regex"));

static REGEX_CUSTOM_PROPERTY_IDENT: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"(^|[^-_a-zA-Z0-9])--([_a-zA-Z][-_a-zA-Z0-9]*)").expect("Invalid regex")
});

struct PresentationalDependencyHashUpdate {
  start: u32,
  end: u32,
  content: String,
}

pub struct CssModuleParser<'parser_and_generator, 'context> {
  generator_options: &'parser_and_generator CssModuleGeneratorOptions,
  parser_options: &'parser_and_generator CssModuleParserOptions,
  source: BoxSource,
  source_code: String,
  cached_source_code: OnceCell<Arc<String>>,
  mode: css_module_lexer::Mode,
  resource_data: &'context ResourceData,
  resource_path: Option<String>,
  compiler_options: &'context CompilerOptions,
  build_info: &'context mut BuildInfo,
  build_meta: &'context mut BuildMeta,
  loaders: &'context [BoxLoader],
  dependencies: Vec<Box<dyn Dependency>>,
  presentational_dependencies: Vec<BoxDependencyTemplate>,
  code_generation_dependencies: Vec<BoxModuleDependency>,
  diagnostics: Vec<Diagnostic>,
  css_exports: Option<CssExports>,
  css_local_names: Option<FxHashMap<String, String>>,
  local_ident_hash_digest: Option<HashDigest>,
  local_ident_hash_digest_length: Option<usize>,
  local_ident_hash_function: Option<HashFunction>,
  local_ident_hash_salt: Option<HashSalt>,
}

impl<'parser_and_generator, 'context> CssModuleParser<'parser_and_generator, 'context> {
  pub fn new(
    generator_options: &'parser_and_generator CssModuleGeneratorOptions,
    parser_options: &'parser_and_generator CssModuleParserOptions,
    parse_context: ParseContext<'context>,
  ) -> Self {
    let ParseContext {
      source,
      module_type,
      resource_data,
      compiler_options,
      build_info,
      build_meta,
      loaders,
      module_match_resource,
      ..
    } = parse_context;

    let source = remove_bom(source);
    let source_code = source.source().into_string_lossy().to_string();
    let resource_data = module_match_resource.unwrap_or(resource_data);
    let resource_path = resource_data.path().map(|path| path.as_str().to_string());
    let mode = Self::mode(module_type, resource_path.as_deref());

    Self {
      generator_options,
      parser_options,
      source,
      source_code,
      cached_source_code: OnceCell::new(),
      mode,
      resource_data,
      resource_path,
      compiler_options,
      build_info,
      build_meta,
      loaders,
      dependencies: vec![],
      presentational_dependencies: vec![],
      code_generation_dependencies: vec![],
      diagnostics: vec![],
      css_exports: None,
      css_local_names: None,
      local_ident_hash_digest: generator_options
        .local_ident_hash_digest
        .as_deref()
        .map(Into::into),
      local_ident_hash_digest_length: generator_options
        .local_ident_hash_digest_length
        .map(|len| len as usize),
      local_ident_hash_function: generator_options
        .local_ident_hash_function
        .as_deref()
        .map(Into::into),
      local_ident_hash_salt: generator_options
        .local_ident_hash_salt
        .clone()
        .map(Some)
        .map(Into::into),
    }
  }

  pub async fn parse(mut self) -> Result<TWithDiagnosticArray<ParseResult>> {
    self.prepare_build_meta();

    let source_code = self.source_code.clone();
    let (deps, warnings) = css_module_lexer::collect_dependencies(&source_code, self.mode);
    let export_names = self.collect_export_names(&deps);
    let module_hash = self.get_css_local_ident_module_hash(&export_names);

    for dependency in deps {
      self.handle_dependency(dependency, &module_hash).await?;
    }
    self.handle_warnings(warnings);

    self.build_info.css_exports = self.css_exports.take();
    self.build_info.css_local_names = self.css_local_names.take();

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
        self.loaders,
      )),
    )
  }

  fn prepare_build_meta(&mut self) {
    self.build_info.strict = true;
    self.build_meta.exports_type = if self.named_exports() {
      BuildMetaExportsType::Namespace
    } else {
      BuildMetaExportsType::Default
    };
    self.build_meta.default_object = if self.named_exports() {
      BuildMetaDefaultObject::False
    } else {
      BuildMetaDefaultObject::Redirect
    };
  }

  fn convention(&self) -> &CssExportsConvention {
    self
      .generator_options
      .exports_convention
      .as_ref()
      .expect("should have convention for module_type css/auto, css/global or css/module")
  }

  fn local_ident_name(&self) -> &LocalIdentName {
    self
      .generator_options
      .local_ident_name
      .as_ref()
      .expect("should have local_ident_name for module_type css/auto, css/global or css/module")
  }

  fn exports_only(&self) -> bool {
    self
      .generator_options
      .exports_only
      .expect("should have exports_only")
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

  fn animation(&self) -> bool {
    self.parser_options.animation.unwrap_or(true)
  }

  fn custom_idents(&self) -> bool {
    self.parser_options.custom_idents.unwrap_or(true)
  }

  fn dashed_idents(&self) -> bool {
    self.parser_options.dashed_idents.unwrap_or(true)
  }

  fn mode(module_type: &ModuleType, resource_path: Option<&str>) -> css_module_lexer::Mode {
    match module_type {
      ModuleType::CssModule => css_module_lexer::Mode::Local,
      ModuleType::CssGlobal => css_module_lexer::Mode::Global,
      ModuleType::CssAuto
        if resource_path.is_some_and(|resource_path| REGEX_IS_MODULES.is_match(resource_path)) =>
      {
        css_module_lexer::Mode::Local
      }
      _ => css_module_lexer::Mode::Css,
    }
  }

  fn get_source_code(&self) -> Arc<String> {
    self
      .cached_source_code
      .get_or_init(|| Arc::new(self.source_code.clone()))
      .clone()
  }

  fn collect_export_names(
    &self,
    deps: &[css_module_lexer::Dependency<'_>],
  ) -> (
    Vec<String>,
    Vec<String>,
    Vec<PresentationalDependencyHashUpdate>,
  ) {
    let mut export_dependency_names = Vec::new();
    let mut graph_export_name_set = FxHashSet::default();
    let mut presentational_dependency_hash_updates = Vec::new();

    for dependency in deps {
      match dependency {
        css_module_lexer::Dependency::LocalClass { name, .. }
        | css_module_lexer::Dependency::LocalId { name, .. } => {
          let (_prefix, name) = name.split_at(1);
          self.collect_export_name(
            name,
            &mut graph_export_name_set,
            &mut export_dependency_names,
          );
        }
        css_module_lexer::Dependency::LocalKeyframes { name, .. }
        | css_module_lexer::Dependency::LocalKeyframesDecl { name, .. }
          if self.animation() =>
        {
          self.collect_export_name(
            name,
            &mut graph_export_name_set,
            &mut export_dependency_names,
          );
        }
        css_module_lexer::Dependency::LocalCounterStyle { name, .. }
        | css_module_lexer::Dependency::LocalCounterStyleDecl { name, .. }
        | css_module_lexer::Dependency::LocalFontPalette { name, .. }
        | css_module_lexer::Dependency::LocalFontPaletteDecl { name, .. }
          if self.custom_idents() =>
        {
          self.collect_export_name(
            name,
            &mut graph_export_name_set,
            &mut export_dependency_names,
          );
        }
        css_module_lexer::Dependency::LocalVar { name, .. }
        | css_module_lexer::Dependency::LocalVarDecl { name, .. }
        | css_module_lexer::Dependency::LocalPropertyDecl { name, .. }
          if self.dashed_idents() =>
        {
          self.collect_export_name(
            name,
            &mut graph_export_name_set,
            &mut export_dependency_names,
          );
        }
        css_module_lexer::Dependency::ICSSExportValue { prop: name, .. } => {
          self.collect_export_name(
            name,
            &mut graph_export_name_set,
            &mut export_dependency_names,
          );
        }
        css_module_lexer::Dependency::Replace { content, range } => {
          presentational_dependency_hash_updates.push(PresentationalDependencyHashUpdate {
            start: range.start,
            end: range.end + 1,
            content: content.to_string(),
          });
        }
        _ => {}
      }
    }

    if self.dashed_idents()
      && let Some(convention) = self.generator_options.exports_convention.as_ref()
    {
      for captures in REGEX_CUSTOM_PROPERTY_IDENT.captures_iter(&self.source_code) {
        if let Some(name) = captures.get(2) {
          let name = name.as_str().to_string();
          for convention_name in export_locals_convention(&name, convention) {
            graph_export_name_set.insert(convention_name);
          }
          export_dependency_names.push(name);
        }
      }
    }

    (
      export_dependency_names,
      graph_export_name_set.into_iter().collect(),
      presentational_dependency_hash_updates,
    )
  }

  fn collect_export_name(
    &self,
    name: &str,
    graph_export_name_set: &mut FxHashSet<String>,
    export_dependency_names: &mut Vec<String>,
  ) {
    if let Some(convention) = self.generator_options.exports_convention.as_ref() {
      let name = unescape(name).into_owned();
      for convention_name in export_locals_convention(&name, convention) {
        graph_export_name_set.insert(convention_name);
      }
      export_dependency_names.push(name);
    }
  }

  async fn handle_dependency(
    &mut self,
    dependency: css_module_lexer::Dependency<'_>,
    module_hash: &str,
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
          .handle_local_ident_declaration(name, range.start + 1, range.end, module_hash)
          .await
      }
      css_module_lexer::Dependency::LocalKeyframes { name, range, .. } => {
        if !self.animation() {
          return Ok(());
        }
        self
          .handle_local_ident_usage(name, range, module_hash)
          .await
      }
      css_module_lexer::Dependency::LocalKeyframesDecl { name, range, .. } => {
        if !self.animation() {
          return Ok(());
        }
        self
          .handle_local_ident_declaration(name, range.start, range.end, module_hash)
          .await
      }
      css_module_lexer::Dependency::LocalCounterStyle { name, range, .. }
      | css_module_lexer::Dependency::LocalFontPalette { name, range, .. } => {
        if !self.custom_idents() {
          return Ok(());
        }
        self
          .handle_local_ident_usage(name, range, module_hash)
          .await
      }
      css_module_lexer::Dependency::LocalCounterStyleDecl { name, range, .. }
      | css_module_lexer::Dependency::LocalFontPaletteDecl { name, range, .. } => {
        if !self.custom_idents() {
          return Ok(());
        }
        self
          .handle_local_ident_declaration(name, range.start, range.end, module_hash)
          .await
      }
      css_module_lexer::Dependency::LocalVar { name, range, .. } => {
        if !self.dashed_idents() {
          return Ok(());
        }
        self
          .handle_local_ident_usage(name, range, module_hash)
          .await
      }
      css_module_lexer::Dependency::LocalVarDecl { name, range, .. }
      | css_module_lexer::Dependency::LocalPropertyDecl { name, range, .. } => {
        if !self.dashed_idents() {
          return Ok(());
        }
        self
          .handle_local_ident_declaration(name, range.start, range.end, module_hash)
          .await
      }
      css_module_lexer::Dependency::Composes {
        local_classes,
        names,
        from,
        range,
      } => self.handle_composes(
        local_classes.into_iter().collect(),
        names.into_iter().collect(),
        from,
        range,
      ),
      css_module_lexer::Dependency::ICSSExportValue { prop, value } => {
        self.handle_icss_export_value(prop, value);
        Ok(())
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

    let request = self.replace_request_prefix(request, &range);
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

    if !self
      .should_resolve_import(request, media, supports, layer)
      .await
    {
      return Ok(());
    }

    let request = self.replace_request_prefix(request, &range);
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

  async fn should_resolve_import(
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
          resource_path: self.resource_path.clone().unwrap_or_default(),
          supports: supports.map(|s| s.to_string()),
          layer: layer.map(|s| s.to_string()),
        };
        (f(args).await).unwrap_or(true)
      }
    }
  }

  fn replace_request_prefix<'a>(
    &mut self,
    request: &'a str,
    range: &css_module_lexer::Range,
  ) -> &'a str {
    let source_code = self.get_source_code();
    replace_module_request_prefix(
      request,
      &mut self.diagnostics,
      || source_code.clone(),
      range.start,
      range.end,
    )
  }

  async fn handle_local_ident_usage(
    &mut self,
    name: &str,
    range: css_module_lexer::Range,
    module_hash: &str,
  ) -> Result<()> {
    let name = unescape(name);
    let (local_ident, convention_names) = self
      .resolve_local_ident_and_update_exports(&name, module_hash)
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

  async fn handle_local_ident_declaration(
    &mut self,
    name: &str,
    start: u32,
    end: u32,
    module_hash: &str,
  ) -> Result<()> {
    let name = unescape(name);
    let (local_ident, convention_names) = self
      .resolve_local_ident_and_update_exports(&name, module_hash)
      .await?;

    let local_names = self.css_local_names.get_or_insert_default();
    local_names.insert(name.into_owned(), local_ident.clone());

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

  fn handle_composes(
    &mut self,
    local_classes: Vec<&str>,
    names: Vec<&str>,
    from: Option<&str>,
    range: css_module_lexer::Range,
  ) -> Result<()> {
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

    let convention = *self.convention();
    let exports = self.css_exports.get_or_insert_default();
    for name in names {
      for local_class in local_classes.iter() {
        let convention_names = export_locals_convention(&name, &convention);
        let convention_local_class = export_locals_convention(local_class, &convention);

        for (convention_name, local_class) in
          convention_names.into_iter().zip(convention_local_class)
        {
          if let Some(existing) = exports.get(name.as_str())
            && from.is_none()
          {
            let existing = existing.clone();
            exports
              .get_mut(local_class.as_str())
              .expect("composes local class must already added to exports")
              .extend(existing);
          } else {
            exports
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
    Ok(())
  }

  fn handle_icss_export_value(&mut self, prop: &str, value: &str) {
    let convention = *self.convention();
    let exports = self.css_exports.get_or_insert_default();
    let convention_names = export_locals_convention(prop, &convention);
    let value = REGEX_IS_COMMENTS.replace_all(value, "");
    for name in convention_names.iter() {
      update_css_exports(
        exports,
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

  async fn resolve_local_ident_and_update_exports(
    &mut self,
    name: &str,
    module_hash: &str,
  ) -> Result<(String, Vec<String>)> {
    let local_ident = LocalIdentOptions::new(
      self.resource_data,
      &self.source_code,
      module_hash,
      self.local_ident_name(),
      self.compiler_options,
      self.local_ident_hash_digest.as_ref(),
      self.local_ident_hash_digest_length,
      self.local_ident_hash_function.as_ref(),
      self.local_ident_hash_salt.as_ref(),
    )
    .get_local_ident(name)
    .await?;
    let convention = *self.convention();
    let exports = self.css_exports.get_or_insert_default();
    let convention_names = export_locals_convention(name, &convention);
    for convention_name in convention_names.iter() {
      update_css_exports(
        exports,
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

  fn handle_warnings(&mut self, warnings: Vec<css_module_lexer::Warning>) {
    for warning in warnings {
      let range = warning.range();
      let error = css_parsing_traceable_error(
        self.get_source_code(),
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

  fn get_css_local_ident_module_hash(
    &self,
    (export_dependency_names, graph_export_names, presentational_dependency_hash_updates): &(
      Vec<String>,
      Vec<String>,
      Vec<PresentationalDependencyHashUpdate>,
    ),
  ) -> String {
    let local_ident_name = self
      .generator_options
      .local_ident_name
      .as_ref()
      .map(|local_ident_name| local_ident_name.template.as_str());
    let relative_resource = make_paths_relative(
      &self.compiler_options.context,
      self.resource_data.resource(),
    );
    let should_use_resource_hash = local_ident_name.is_some_and(|local_ident_name| {
      (local_ident_name.contains("[hash") || local_ident_name.contains("[fullhash"))
        && !relative_resource.contains(['?', '#'])
    }) && self.generator_options.local_ident_hash_salt.is_none();
    if should_use_resource_hash {
      let mut hasher = RspackHash::with_salt(
        &self.compiler_options.output.hash_function,
        &self.compiler_options.output.hash_salt,
      );
      hasher.write(relative_resource.as_bytes());
      let hash = hasher
        .digest(&self.compiler_options.output.hash_digest)
        .rendered(self.compiler_options.output.hash_digest_length)
        .to_string();
      return LEADING_DIGIT_REGEX.replace(&hash, "_${1}").into_owned();
    }

    let hash_function = self
      .local_ident_hash_function
      .as_ref()
      .unwrap_or(&self.compiler_options.output.hash_function);
    let build_hash = self.get_build_hash(hash_function);
    let graph_hash = self.get_graph_hash(hash_function, &relative_resource, graph_export_names);

    let mut hasher = RspackHash::new(hash_function);
    hasher.write(build_hash.as_bytes());
    if self.exports_only() {
      hasher.write(b"javascript");
    } else {
      hasher.write(b"javascript");
      hasher.write(b"css");
    }
    hasher.write(if self.es_module() { b"true" } else { b"false" });
    hasher.write(if self.exports_only() {
      b"true"
    } else {
      b"false"
    });
    hasher.write(graph_hash.as_bytes());
    self.update_hash_with_presentational_dependencies(
      &mut hasher,
      presentational_dependency_hash_updates,
    );
    self.update_hash_with_export_dependencies(&mut hasher, export_dependency_names);
    hasher.digest(&HashDigest::Hex).rendered(20).to_string()
  }

  fn get_build_hash(&self, hash_function: &HashFunction) -> String {
    let mut hasher = RspackHash::new(hash_function);
    hasher.write(b"source");
    hasher.write(b"RawSource");
    hasher.write(self.source_code.as_bytes());
    hasher.write(b"meta");
    hasher.write(br#"{"isCssModule":true,"exportsType":"namespace","defaultObject":false}"#);
    hasher.digest(&HashDigest::Hex).encoded().to_string()
  }

  fn get_graph_hash(
    &self,
    hash_function: &HashFunction,
    relative_resource: &str,
    graph_export_names: &[String],
  ) -> String {
    let mut graph_exports = graph_export_names.to_vec();
    graph_exports.sort();

    let mut hasher = RspackHash::new(hash_function);
    hasher.write(relative_resource.as_bytes());
    hasher.write(b"false");
    for name in graph_exports {
      hasher.write(name.as_bytes());
      hasher.write(b"2truefalse");
    }
    hasher.write(b"*side effects only*2undefinedfalse");
    hasher.write(b"null2falsefalse");
    hasher.digest(&HashDigest::Hex).encoded().to_string()
  }

  fn update_hash_with_presentational_dependencies(
    &self,
    hasher: &mut RspackHash,
    presentational_dependency_hash_updates: &[PresentationalDependencyHashUpdate],
  ) {
    let mut itoa_buffer = itoa::Buffer::new();
    for update in presentational_dependency_hash_updates {
      hasher.write(itoa_buffer.format(update.start).as_bytes());
      hasher.write(b",");
      hasher.write(itoa_buffer.format(update.end).as_bytes());
      hasher.write(b"|");
      hasher.write(update.content.as_bytes());
    }
  }

  fn update_hash_with_export_dependencies(
    &self,
    hasher: &mut RspackHash,
    export_dependency_names: &[String],
  ) {
    let local_ident_name = self
      .generator_options
      .local_ident_name
      .as_ref()
      .map(|local_ident_name| local_ident_name.template.as_str());
    for name in export_dependency_names {
      let convention_names = export_locals_convention(name, self.convention());
      let convention_names =
        serde_json::to_string(&convention_names).expect("css export names should be serializable");
      if let Some(local_ident_name) = local_ident_name {
        let local_ident_name =
          serde_json::to_string(local_ident_name).expect("local ident name should be serializable");
        hasher.write(b"exportsConvention|");
        hasher.write(convention_names.as_bytes());
        hasher.write(b"|localIdentName|");
        hasher.write(local_ident_name.as_bytes());
      }
    }
  }
}

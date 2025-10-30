use std::{
  borrow::Cow,
  sync::{Arc, LazyLock},
};

use indexmap::{IndexMap, IndexSet};
use once_cell::sync::OnceCell;
use regex::Regex;
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsMap, AsOption, AsVec},
};
use rspack_core::{
  BoxDependencyTemplate, BoxModuleDependency, BuildMetaDefaultObject, BuildMetaExportsType,
  ChunkGraph, Compilation, ConstDependency, CssExportsConvention, Dependency, DependencyId,
  DependencyRange, DependencyType, GenerateContext, LocalIdentName, Module, ModuleGraph,
  ModuleIdentifier, ModuleInitFragments, ModuleType, NormalModule, ParseContext, ParseResult,
  ParserAndGenerator, PrefetchExportsInfoMode, RuntimeGlobals, RuntimeSpec, SourceType,
  TemplateContext, UsageState,
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics,
  remove_bom,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt},
};
use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, Severity, TWithDiagnosticArray};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::{atom::Atom, ext::DynHash};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  dependency::{
    CssComposeDependency, CssExportDependency, CssImportDependency, CssLayer,
    CssLocalIdentDependency, CssMedia, CssSelfReferenceLocalIdentDependency,
    CssSelfReferenceLocalIdentReplacement, CssSupports, CssUrlDependency,
  },
  utils::{
    LocalIdentOptions, css_modules_exports_to_concatenate_module_string,
    css_modules_exports_to_string, css_parsing_traceable_error, export_locals_convention,
    normalize_url, replace_module_request_prefix, unescape,
  },
};

static REGEX_IS_MODULES: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\.module(s)?\.[^.]+$").expect("Invalid regex"));

static REGEX_IS_COMMENTS: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"/\*[\s\S]*?\*/").expect("Invalid regex"));

pub(crate) static CSS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::Css, SourceType::JavaScript];

pub(crate) static CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST: &[SourceType; 1] =
  &[SourceType::JavaScript];

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CssExport {
  pub ident: String,
  pub from: Option<String>,
  pub id: Option<DependencyId>,
}

pub type CssExports = IndexMap<String, IndexSet<CssExport>>;

fn update_css_exports(exports: &mut CssExports, name: String, css_export: CssExport) -> bool {
  if let Some(existing) = exports.get_mut(&name) {
    existing.insert(css_export)
  } else {
    exports
      .insert(name, IndexSet::from_iter([css_export]))
      .is_none()
  }
}

#[cacheable]
#[derive(Debug)]
pub struct CssParserAndGenerator {
  pub convention: Option<CssExportsConvention>,
  pub local_ident_name: Option<LocalIdentName>,
  pub exports_only: bool,
  pub named_exports: bool,
  pub es_module: bool,
  pub url: bool,
  #[cacheable(with=AsOption<AsMap<AsCacheable, AsVec>>)]
  pub exports: Option<CssExports>,
  pub local_names: Option<FxHashMap<String, String>>,
  pub hot: bool,
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl ParserAndGenerator for CssParserAndGenerator {
  fn source_types(&self, _module: &dyn Module, _module_graph: &ModuleGraph) -> &[SourceType] {
    if self.exports_only {
      CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST
    } else {
      CSS_MODULE_SOURCE_TYPE_LIST
    }
  }

  fn size(&self, module: &dyn Module, source_type: Option<&SourceType>) -> f64 {
    match source_type.unwrap_or(&SourceType::Css) {
      SourceType::JavaScript => 42.0,
      SourceType::Css => module.source().map_or(0, |source| source.size()) as f64,
      _ => unreachable!(),
    }
  }

  async fn parse<'a>(
    &mut self,
    parse_context: ParseContext<'a>,
  ) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      module_type,
      resource_data,
      compiler_options,
      build_info,
      build_meta,
      loaders,
      ..
    } = parse_context;

    build_info.strict = true;
    build_meta.exports_type = if self.named_exports {
      BuildMetaExportsType::Namespace
    } else {
      BuildMetaExportsType::Default
    };
    build_meta.default_object = if self.named_exports {
      BuildMetaDefaultObject::False
    } else {
      BuildMetaDefaultObject::Redirect
    };

    let source = remove_bom(source);
    let source_code = source.source().into_string_lossy();
    let resource_path = resource_data.path();
    let cached_source_code = OnceCell::new();
    let get_source_code = || {
      let s = cached_source_code.get_or_init(|| Arc::new(source_code.to_string()));
      s.clone()
    };

    let mode = match module_type {
      ModuleType::CssModule => css_module_lexer::Mode::Local,
      ModuleType::CssAuto
        if resource_path.is_some()
          && REGEX_IS_MODULES.is_match(
            resource_path
              .as_ref()
              .expect("should have resource_path for module_type css/auto")
              .as_str(),
          ) =>
      {
        css_module_lexer::Mode::Local
      }
      _ => css_module_lexer::Mode::Css,
    };

    let mut diagnostics: Vec<Diagnostic> = vec![];
    let mut dependencies: Vec<Box<dyn Dependency>> = vec![];
    let mut presentational_dependencies: Vec<BoxDependencyTemplate> = vec![];
    let mut code_generation_dependencies: Vec<BoxModuleDependency> = vec![];

    let (deps, warnings) = css_module_lexer::collect_dependencies(&source_code, mode);
    for dependency in deps {
      match dependency {
        css_module_lexer::Dependency::Url {
          request,
          range,
          kind,
        } => {
          if request.trim().is_empty() {
            continue;
          }
          if !self.url {
            continue;
          }

          let request = replace_module_request_prefix(
            request,
            &mut diagnostics,
            get_source_code,
            range.start,
            range.end,
          );
          let request = normalize_url(request);
          let dep = Box::new(CssUrlDependency::new(
            request,
            DependencyRange::new(range.start, range.end),
            matches!(kind, css_module_lexer::UrlRangeKind::Function),
          ));
          dependencies.push(dep.clone());
          code_generation_dependencies.push(dep);
        }
        css_module_lexer::Dependency::Import {
          request,
          range,
          media,
          supports,
          layer,
        } => {
          if request.is_empty() {
            presentational_dependencies.push(Box::new(ConstDependency::new(
              (range.start, range.end).into(),
              "".into(),
              None,
            )));
            continue;
          }
          let request = replace_module_request_prefix(
            request,
            &mut diagnostics,
            get_source_code,
            range.start,
            range.end,
          );
          dependencies.push(Box::new(CssImportDependency::new(
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
        }
        css_module_lexer::Dependency::Replace { content, range } => presentational_dependencies
          .push(Box::new(ConstDependency::new(
            (range.start, range.end).into(),
            content.into(),
            None,
          ))),
        css_module_lexer::Dependency::LocalClass { name, range, .. }
        | css_module_lexer::Dependency::LocalId { name, range, .. } => {
          let (_prefix, name) = name.split_at(1); // split '#' or '.'
          let name = unescape(name);

          let local_ident = LocalIdentOptions::new(
            resource_data,
            self
              .local_ident_name
              .as_ref()
              .expect("should have local_ident_name for module_type css/auto or css/module"),
            compiler_options,
          )
          .get_local_ident(&name)
          .await?;
          let convention = self
            .convention
            .as_ref()
            .expect("should have local_ident_name for module_type css/auto or css/module");
          let exports = self.exports.get_or_insert_default();
          let convention_names = export_locals_convention(&name, convention);
          for name in convention_names.iter() {
            update_css_exports(
              exports,
              name.to_owned(),
              CssExport {
                ident: local_ident.clone(),
                from: None,
                id: None,
              },
            );
          }

          let local_names = self.local_names.get_or_insert_default();
          local_names.insert(name.into_owned(), local_ident.clone());

          dependencies.push(Box::new(CssLocalIdentDependency::new(
            local_ident,
            convention_names,
            range.start + 1,
            range.end,
          )));
        }
        css_module_lexer::Dependency::LocalKeyframes { name, range, .. } => {
          let name = unescape(name);
          let local_ident = LocalIdentOptions::new(
            resource_data,
            self
              .local_ident_name
              .as_ref()
              .expect("should have local_ident_name for module_type css/auto or css/module"),
            compiler_options,
          )
          .get_local_ident(&name)
          .await?;
          let exports = self.exports.get_or_insert_default();
          let convention = self
            .convention
            .as_ref()
            .expect("should have local_ident_name for module_type css/auto or css/module");
          let convention_names = export_locals_convention(&name, convention);
          for name in convention_names.iter() {
            update_css_exports(
              exports,
              name.to_owned(),
              CssExport {
                ident: local_ident.clone(),
                from: None,
                id: None,
              },
            );
          }
          dependencies.push(Box::new(CssSelfReferenceLocalIdentDependency::new(
            convention_names,
            vec![CssSelfReferenceLocalIdentReplacement {
              local_ident: local_ident.clone(),
              range: (range.start, range.end).into(),
            }],
          )));
        }
        css_module_lexer::Dependency::LocalKeyframesDecl { name, range, .. } => {
          let name = unescape(name);
          let local_ident = LocalIdentOptions::new(
            resource_data,
            self
              .local_ident_name
              .as_ref()
              .expect("should have local_ident_name for module_type css/auto or css/module"),
            compiler_options,
          )
          .get_local_ident(&name)
          .await?;
          let exports = self.exports.get_or_insert_default();
          let convention = self
            .convention
            .as_ref()
            .expect("should have local_ident_name for module_type css/auto or css/module");
          let convention_names = export_locals_convention(&name, convention);
          for name in convention_names.iter() {
            update_css_exports(
              exports,
              name.to_owned(),
              CssExport {
                ident: local_ident.clone(),
                from: None,
                id: None,
              },
            );
          }

          let local_names = self.local_names.get_or_insert_default();
          local_names.insert(name.into_owned(), local_ident.clone());

          dependencies.push(Box::new(CssLocalIdentDependency::new(
            local_ident.clone(),
            convention_names,
            range.start,
            range.end,
          )));
        }
        css_module_lexer::Dependency::Composes {
          local_classes,
          names,
          from,
          range,
        } => {
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
            dependencies.push(Box::new(dep));
          } else if from.is_none() {
            dependencies.push(Box::new(CssSelfReferenceLocalIdentDependency::new(
              names.iter().map(|s| s.to_string()).collect(),
              vec![],
            )));
          }

          let exports = self.exports.get_or_insert_default();
          for name in names {
            for local_class in local_classes.iter() {
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
                    ident: name.to_string(),
                    from: from
                      .filter(|f| *f != "global")
                      .map(|f| f.trim_matches(|c| c == '\'' || c == '"').to_string()),
                    id: dep_id,
                  });
              }
            }
          }
        }
        css_module_lexer::Dependency::ICSSExportValue { prop, value } => {
          let exports = self.exports.get_or_insert_default();
          let convention = self
            .convention
            .as_ref()
            .expect("should have local_ident_name for module_type css/auto or css/module");
          let convention_names = export_locals_convention(prop, convention);
          let value = REGEX_IS_COMMENTS.replace_all(value, "");
          for name in convention_names.iter() {
            update_css_exports(
              exports,
              name.to_owned(),
              CssExport {
                ident: value.to_string(),
                from: None,
                id: None,
              },
            );
          }
          dependencies.push(Box::new(CssExportDependency::new(convention_names)));
        }
        _ => {}
      }
    }
    for warning in warnings {
      let range = warning.range();
      let error = css_parsing_traceable_error(
        get_source_code(),
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
      diagnostics.push(error.into());
    }

    Ok(
      ParseResult {
        dependencies,
        blocks: vec![],
        presentational_dependencies,
        code_generation_dependencies,
        source,
        side_effects_bailout: None,
      }
      .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
        diagnostics,
        loaders,
      )),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  async fn generate(
    &self,
    source: &BoxSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    match generate_context.requested_source_type {
      SourceType::Css => {
        generate_context
          .runtime_requirements
          .insert(RuntimeGlobals::HAS_CSS_MODULES);

        let mut source = ReplaceSource::new(source.clone());
        let compilation = generate_context.compilation;
        let mut init_fragments = ModuleInitFragments::default();
        let mut context = TemplateContext {
          compilation,
          module,
          runtime_requirements: generate_context.runtime_requirements,
          runtime: generate_context.runtime,
          init_fragments: &mut init_fragments,
          concatenation_scope: generate_context.concatenation_scope.take(),
          data: generate_context.data,
        };

        let module_graph = compilation.get_module_graph();
        module.get_dependencies().iter().for_each(|id| {
          let dep = module_graph
            .dependency_by_id(id)
            .expect("should have dependency");

          if let Some(dependency) = dep.as_dependency_code_generation() {
            if let Some(template) = compilation.get_dependency_template(dependency) {
              template.render(dependency, &mut source, &mut context)
            } else {
              panic!(
                "Can not find dependency template of {:?}",
                dependency.dependency_template()
              );
            }
          }
        });

        for conn in module_graph.get_incoming_connections(&module.identifier()) {
          let Some(dep) = module_graph.dependency_by_id(&conn.dependency_id) else {
            continue;
          };

          if matches!(dep.dependency_type(), DependencyType::CssImport) {
            let Some(css_import_dep) = dep.downcast_ref::<CssImportDependency>() else {
              panic!(
                "dependency with type DependencyType::CssImport should only be CssImportDependency"
              );
            };

            if let Some(media) = css_import_dep.media() {
              let media = CssMedia(media.to_string());
              context.data.insert(media);
            }

            if let Some(supports) = css_import_dep.supports() {
              let supports = CssSupports(supports.to_string());
              context.data.insert(supports);
            }

            if let Some(layer) = css_import_dep.layer() {
              context.data.insert(layer.clone());
            }
          }
        }

        if let Some(dependencies) = module.get_presentational_dependencies() {
          dependencies.iter().for_each(|dependency| {
            if let Some(template) = compilation.get_dependency_template(dependency.as_ref()) {
              template.render(dependency.as_ref(), &mut source, &mut context)
            } else {
              panic!(
                "Can not find dependency template of {:?}",
                dependency.dependency_template()
              );
            }
          });
        };

        generate_context.concatenation_scope = context.concatenation_scope.take();

        Ok(source.boxed())
      }
      SourceType::JavaScript => {
        let with_hmr = self.hot;
        let exports = if generate_context.concatenation_scope.is_some() {
          // currently this is dead branch, as css module will never be concatenated expect exportsOnly
          let mut concate_source = ConcatSource::default();
          if let Some(ref exports) = self.exports {
            let mg = generate_context.compilation.get_module_graph();
            if let Some(local_names) = &self.local_names {
              let unused_exports = get_unused_local_ident(
                exports,
                local_names,
                module.identifier(),
                generate_context.runtime,
                &mg,
              );
              generate_context.data.insert(unused_exports);
            }
            let exports =
              get_used_exports(exports, module.identifier(), generate_context.runtime, &mg);

            css_modules_exports_to_concatenate_module_string(
              exports,
              module,
              generate_context,
              &mut concate_source,
            )?;
          }
          return Ok(concate_source.boxed());
        } else {
          let mg = generate_context.compilation.get_module_graph();
          let exports_info =
            mg.get_prefetched_exports_info(&module.identifier(), PrefetchExportsInfoMode::Default);
          let (ns_obj, left, right) = if self.es_module
            && exports_info
              .other_exports_info()
              .get_used(generate_context.runtime)
              != UsageState::Unused
          {
            (RuntimeGlobals::MAKE_NAMESPACE_OBJECT.name(), "(", ")")
          } else {
            ("", "", "")
          };
          if let Some(exports) = &self.exports {
            if let Some(local_names) = &self.local_names {
              let unused_exports = get_unused_local_ident(
                exports,
                local_names,
                module.identifier(),
                generate_context.runtime,
                &mg,
              );
              generate_context.data.insert(unused_exports);
            }

            let exports =
              get_used_exports(exports, module.identifier(), generate_context.runtime, &mg);

            css_modules_exports_to_string(
              exports,
              module,
              generate_context.compilation,
              generate_context.runtime_requirements,
              ns_obj,
              left,
              right,
              with_hmr,
            )?
          } else {
            format!(
              "{}{}module.exports = {{}}{};\n{}",
              ns_obj,
              left,
              right,
              if with_hmr {
                "module.hot.accept();\n"
              } else {
                Default::default()
              }
            )
          }
        };
        generate_context
          .runtime_requirements
          .insert(RuntimeGlobals::MODULE);
        if self.es_module {
          generate_context
            .runtime_requirements
            .insert(RuntimeGlobals::MAKE_NAMESPACE_OBJECT);
        }
        Ok(RawStringSource::from(exports).boxed())
      }
      _ => panic!(
        "Unsupported source type: {:?}",
        generate_context.requested_source_type
      ),
    }
  }

  fn get_concatenation_bailout_reason(
    &self,
    _module: &dyn rspack_core::Module,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    if self.exports_only {
      None
    } else {
      // CSS Module cannot be concatenated as it must appear in css chunk, if it's
      // concatenated, it will be removed from module graph
      Some("Module Concatenation is not implemented for CssParserAndGenerator".into())
    }
  }

  async fn get_runtime_hash(
    &self,
    _module: &NormalModule,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    self.es_module.dyn_hash(&mut hasher);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

fn get_used_exports<'a>(
  exports: &'a CssExports,
  identifier: ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
  mg: &ModuleGraph,
) -> IndexMap<&'a str, &'a IndexSet<CssExport>> {
  let exports_info =
    mg.get_prefetched_exports_info_optional(&identifier, PrefetchExportsInfoMode::Default);

  exports
    .iter()
    .filter(|(name, _)| {
      let export_info = exports_info
        .as_ref()
        .map(|info| info.get_read_only_export_info(&Atom::from(name.as_str())));

      if let Some(export_info) = export_info {
        export_info.get_used(runtime) != UsageState::Unused
      } else {
        true
      }
    })
    .map(|(name, exports)| (name.as_str(), exports))
    .collect()
}

#[derive(Debug, Clone)]
pub struct CodeGenerationDataUnusedLocalIdent {
  pub(crate) idents: FxHashSet<String>,
}

fn get_unused_local_ident(
  exports: &CssExports,
  local_names: &FxHashMap<String, String>,
  identifier: ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
  mg: &ModuleGraph,
) -> CodeGenerationDataUnusedLocalIdent {
  let exports_names = exports
    .iter()
    .map(|(name, _)| Atom::from(name.as_str()))
    .collect::<Vec<_>>();
  let exports_info =
    mg.get_prefetched_exports_info_optional(&identifier, PrefetchExportsInfoMode::Default);

  CodeGenerationDataUnusedLocalIdent {
    idents: exports_names
      .iter()
      .filter(|name| {
        let export_info = exports_info
          .as_ref()
          .map(|info| info.get_read_only_export_info(name));

        if let Some(export_info) = export_info {
          matches!(export_info.get_used(runtime), UsageState::Unused)
        } else {
          false
        }
      })
      .filter_map(|export_name| local_names.get(export_name.as_str()).cloned())
      .collect(),
  }
}

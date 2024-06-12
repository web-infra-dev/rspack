use std::collections::HashMap;

use indexmap::{IndexMap, IndexSet};
use once_cell::sync::Lazy;
use regex::Regex;
use rkyv::{from_bytes, to_bytes, AlignedVec};
use rspack_core::{
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics,
  rspack_sources::{BoxSource, ConcatSource, RawSource, ReplaceSource, Source, SourceExt},
  BuildExtraDataType, BuildMetaDefaultObject, BuildMetaExportsType, ChunkGraph, ConstDependency,
  CssExportsConvention, Dependency, DependencyTemplate, ErrorSpan, GenerateContext, LocalIdentName,
  Module, ModuleDependency, ModuleGraph, ModuleIdentifier, ModuleType, ParseContext, ParseResult,
  ParserAndGenerator, RuntimeSpec, SourceType, TemplateContext, UsageState,
};
use rspack_core::{ModuleInitFragments, RuntimeGlobals};
use rspack_error::{
  miette::Diagnostic, IntoTWithDiagnosticArray, Result, RspackSeverity, TWithDiagnosticArray,
};

use crate::utils::export_locals_convention;
use crate::utils::{css_modules_exports_to_string, LocalIdentOptions};
use crate::{
  dependency::{
    CssComposeDependency, CssExportDependency, CssImportDependency, CssLocalIdentDependency,
    CssUrlDependency,
  },
  utils::{
    css_modules_exports_to_concatenate_module_string, css_parsing_traceable_error, normalize_url,
    replace_module_request_prefix,
  },
};

static REGEX_IS_MODULES: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.module(s)?\.[^.]+$").expect("Invalid regex"));

pub(crate) static CSS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::JavaScript, SourceType::Css];

pub(crate) static CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST: &[SourceType; 1] =
  &[SourceType::JavaScript];

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(PartialEq, Eq, Hash))]
pub struct CssExport {
  pub ident: String,
  pub from: Option<String>,
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

#[derive(Debug)]
pub struct CssParserAndGenerator {
  pub convention: Option<CssExportsConvention>,
  pub local_ident_name: Option<LocalIdentName>,
  pub exports_only: bool,
  pub named_exports: bool,
  pub es_module: bool,
  pub exports: Option<CssExports>,
}

impl ParserAndGenerator for CssParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    if self.exports_only {
      CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST
    } else {
      CSS_MODULE_SOURCE_TYPE_LIST
    }
  }

  fn size(&self, module: &dyn Module, source_type: &SourceType) -> f64 {
    match source_type {
      SourceType::JavaScript => 42.0,
      SourceType::Css => module.original_source().map_or(0, |source| source.size()) as f64,
      _ => unreachable!(),
    }
  }

  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
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

    let source_code = source.source();
    let resource_path = &resource_data.resource_path;

    let mode = match module_type {
      ModuleType::CssModule => css_module_lexer::Mode::Local,
      ModuleType::CssAuto
        if REGEX_IS_MODULES.is_match(resource_path.to_string_lossy().as_ref()) =>
      {
        css_module_lexer::Mode::Local
      }
      _ => css_module_lexer::Mode::Css,
    };

    let mut diagnostics: Vec<Box<dyn Diagnostic + Send + Sync + 'static>> = vec![];
    let mut dependencies: Vec<Box<dyn Dependency>> = vec![];
    let mut presentational_dependencies: Vec<Box<dyn DependencyTemplate>> = vec![];
    let mut code_generation_dependencies: Vec<Box<dyn ModuleDependency>> = vec![];

    let (deps, warnings) = css_module_lexer::collect_dependencies(&source_code, mode);
    for dependency in deps {
      match dependency {
        css_module_lexer::Dependency::Url {
          request,
          range,
          kind,
        } => {
          if request.is_empty() {
            continue;
          }
          let request = replace_module_request_prefix(
            request,
            &mut diagnostics,
            &source_code,
            range.start,
            range.end,
          );
          let request = normalize_url(request);
          let dep = Box::new(CssUrlDependency::new(
            request,
            Some(ErrorSpan::new(range.start, range.end)),
            range.start,
            range.end,
            matches!(kind, css_module_lexer::UrlRangeKind::Function),
          ));
          dependencies.push(dep.clone());
          code_generation_dependencies.push(dep);
        }
        css_module_lexer::Dependency::Import { request, range, .. } => {
          if request.is_empty() {
            presentational_dependencies.push(Box::new(ConstDependency::new(
              range.start,
              range.end,
              "".into(),
              None,
            )));
            continue;
          }
          let request = replace_module_request_prefix(
            request,
            &mut diagnostics,
            &source_code,
            range.start,
            range.end,
          );
          dependencies.push(Box::new(CssImportDependency::new(
            request.to_string(),
            Some(ErrorSpan::new(range.start, range.end)),
            range.start,
            range.end,
          )));
        }
        css_module_lexer::Dependency::Replace { content, range } => presentational_dependencies
          .push(Box::new(ConstDependency::new(
            range.start,
            range.end,
            content.into(),
            None,
          ))),
        css_module_lexer::Dependency::LocalClass { name, range, .. }
        | css_module_lexer::Dependency::LocalId { name, range, .. } => {
          let (prefix, name) = name.split_at(1); // split '#' or '.'
          let local_ident = LocalIdentOptions::new(
            resource_data,
            self
              .local_ident_name
              .as_ref()
              .expect("should have local_ident_name for module_type css/auto or css/module"),
            compiler_options,
          )
          .get_local_ident(name);
          dependencies.push(Box::new(CssLocalIdentDependency::new(
            name.to_string(),
            format!("{prefix}{local_ident}"),
            range.start,
            range.end,
          )));
          let exports = self.exports.get_or_insert_default();
          let convention = self
            .convention
            .as_ref()
            .expect("should have local_ident_name for module_type css/auto or css/module");
          for name in export_locals_convention(name, convention) {
            update_css_exports(
              exports,
              name,
              CssExport {
                ident: local_ident.clone(),
                from: None,
              },
            );
          }
        }
        css_module_lexer::Dependency::LocalKeyframes { name, range, .. }
        | css_module_lexer::Dependency::LocalKeyframesDecl { name, range, .. } => {
          let local_ident = LocalIdentOptions::new(
            resource_data,
            self
              .local_ident_name
              .as_ref()
              .expect("should have local_ident_name for module_type css/auto or css/module"),
            compiler_options,
          )
          .get_local_ident(name);
          dependencies.push(Box::new(CssLocalIdentDependency::new(
            name.to_string(),
            local_ident.clone(),
            range.start,
            range.end,
          )));
          let exports = self.exports.get_or_insert_default();
          let convention = self
            .convention
            .as_ref()
            .expect("should have local_ident_name for module_type css/auto or css/module");
          for name in export_locals_convention(name, convention) {
            update_css_exports(
              exports,
              name,
              CssExport {
                ident: local_ident.clone(),
                from: None,
              },
            );
          }
        }
        css_module_lexer::Dependency::Composes {
          local_classes,
          names,
          from,
          range,
        } => {
          if let Some(from) = from
            && from != "global"
          {
            let from = from.trim_matches(|c| c == '\'' || c == '"');
            dependencies.push(Box::new(CssComposeDependency::new(
              from.to_string(),
              ErrorSpan::new(range.start, range.end),
            )));
          }
          let exports = self.exports.get_or_insert_default();
          for name in names {
            for &local_class in local_classes.iter() {
              if let Some(existing) = exports.get(name) {
                let existing = existing.clone();
                exports
                  .get_mut(local_class)
                  .expect("composes local class must already added to exports")
                  .extend(existing);
              } else {
                exports
                  .get_mut(local_class)
                  .expect("composes local class must already added to exports")
                  .insert(CssExport {
                    ident: name.to_string(),
                    from: from
                      .filter(|f| *f != "global")
                      .map(|f| f.trim_matches(|c| c == '\'' || c == '"').to_string()),
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
          for name in export_locals_convention(prop, convention) {
            dependencies.push(Box::new(CssExportDependency::new(
              name.clone(),
              value.to_string(),
            )));
            update_css_exports(
              exports,
              name,
              CssExport {
                ident: value.to_string(),
                from: None,
              },
            );
          }
        }
        _ => {}
      }
    }
    for warning in warnings {
      let range = warning.range();
      let error = css_parsing_traceable_error(
        source_code.clone(),
        range.start,
        range.end,
        warning.to_string(),
        if matches!(
          warning.kind(),
          css_module_lexer::WarningKind::NotPrecededAtImport
        ) {
          RspackSeverity::Error
        } else {
          RspackSeverity::Warn
        },
      );
      diagnostics.push(Box::new(error));
    }

    Ok(
      ParseResult {
        dependencies,
        blocks: vec![],
        presentational_dependencies,
        code_generation_dependencies,
        source,
        analyze_result: Default::default(),
        side_effects_bailout: None,
      }
      .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
        diagnostics,
        loaders,
      )),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    source: &BoxSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    let result = match generate_context.requested_source_type {
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

        module.get_dependencies().iter().for_each(|id| {
          if let Some(dependency) = compilation
            .get_module_graph()
            .dependency_by_id(id)
            .expect("should have dependency")
            .as_dependency_template()
          {
            dependency.apply(&mut source, &mut context)
          }
        });

        if let Some(dependencies) = module.get_presentational_dependencies() {
          dependencies
            .iter()
            .for_each(|dependency| dependency.apply(&mut source, &mut context));
        };

        generate_context.concatenation_scope = context.concatenation_scope.take();
        Ok(source.boxed())
      }
      SourceType::JavaScript => {
        let exports = if generate_context.concatenation_scope.is_some() {
          let mut concate_source = ConcatSource::default();
          if let Some(ref exports) = self.exports {
            let mg = generate_context.compilation.get_module_graph();

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
          let (ns_obj, left, right) = if self.es_module {
            (RuntimeGlobals::MAKE_NAMESPACE_OBJECT.name(), "(", ")")
          } else {
            ("", "", "")
          };
          if let Some(exports) = &self.exports {
            let mg = generate_context.compilation.get_module_graph();

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
            )?
          } else if generate_context.compilation.options.dev_server.hot {
            format!(
              "module.hot.accept();\n{}{}module.exports = {{}}{};\n",
              ns_obj, left, right
            )
          } else {
            format!("{}{}module.exports = {{}}{};\n", ns_obj, left, right)
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
        Ok(RawSource::from(exports).boxed())
      }
      _ => panic!(
        "Unsupported source type: {:?}",
        generate_context.requested_source_type
      ),
    };

    result
  }

  fn store(&self, extra_data: &mut HashMap<BuildExtraDataType, AlignedVec>) {
    let data = self.exports.to_owned();
    extra_data.insert(
      BuildExtraDataType::CssParserAndGenerator,
      to_bytes::<_, 1024>(&data).expect("Failed to store extra data"),
    );
  }

  fn resume(&mut self, extra_data: &HashMap<BuildExtraDataType, AlignedVec>) {
    if let Some(data) = extra_data.get(&BuildExtraDataType::CssParserAndGenerator) {
      let data = from_bytes::<Option<CssExports>>(data).expect("Failed to resume extra data");
      self.exports = data;
    }
  }

  fn get_concatenation_bailout_reason(
    &self,
    _module: &dyn rspack_core::Module,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<String> {
    Some(String::from(
      "Module Concatenation is not implemented for CssParserAndGenerator",
    ))
  }
}

fn get_used_exports<'a>(
  exports: &'a CssExports,
  identifier: ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
  mg: &ModuleGraph,
) -> IndexMap<&'a str, &'a IndexSet<CssExport>> {
  exports
    .iter()
    .filter(|(name, _)| {
      let export_info = mg.get_read_only_export_info(&identifier, name.as_str().into());

      if let Some(export_info) = export_info {
        !matches!(export_info.get_used(runtime), UsageState::Unused)
      } else {
        true
      }
    })
    .map(|(name, exports)| (name.as_str(), exports))
    .collect()
}

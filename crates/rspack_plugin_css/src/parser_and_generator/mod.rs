use std::{
  borrow::Cow,
  sync::{Arc, LazyLock},
};

use once_cell::sync::OnceCell;
use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  BoxDependencyTemplate, BoxModuleDependency, BuildMetaDefaultObject, BuildMetaExportsType,
  ChunkGraph, Compilation, CompilerOptions, ConstDependency, CssExportType, CssExportsConvention,
  CssModuleGeneratorOptionsNormalized, CssParserImport, CssParserImportContext, Dependency,
  DependencyRange, DependencyType, ExportsInfoArtifact, GenerateContext, LocalIdentName, Module,
  ModuleArgument, ModuleGraph, ModuleIdentifier, ModuleInitFragments, ModuleType, NormalModule,
  ParseContext, ParseResult, ParserAndGenerator, ResourceData, RuntimeGlobals, RuntimeSpec,
  SourceType, TemplateContext, UsageState,
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics,
  remove_bom,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt},
};
pub use rspack_core::{CssExport, CssExports};
use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, Severity, TWithDiagnosticArray};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::{
  atom::Atom,
  ext::DynHash,
  fx_hash::{FxIndexMap, FxIndexSet},
};
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

pub(crate) static CSS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 1] = &[SourceType::Css];

pub(crate) static CSS_MODULE_AND_JS_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::Css, SourceType::JavaScript];

pub(crate) static CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST: &[SourceType; 1] =
  &[SourceType::JavaScript];

fn update_css_exports(exports: &mut CssExports, name: String, css_export: CssExport) -> bool {
  if let Some(existing) = exports.get_mut(&name) {
    existing.insert(css_export)
  } else {
    exports
      .insert(name, FxIndexSet::from_iter([css_export]))
      .is_none()
  }
}

#[cacheable]
#[derive(Debug)]
pub struct CssParserAndGenerator {
  pub generator_options: CssModuleGeneratorOptionsNormalized,
  pub named_exports: bool,
  pub url: bool,
  pub resolve_import: CssParserImport,
  pub hot: bool,
}

impl CssParserAndGenerator {
  pub fn new(
    generator_options: CssModuleGeneratorOptionsNormalized,
    named_exports: bool,
    url: bool,
    resolve_import: CssParserImport,
  ) -> Self {
    Self {
      generator_options,
      named_exports,
      url,
      resolve_import,
      hot: false,
    }
  }

  pub fn convention(&self) -> &CssExportsConvention {
    self.generator_options.convention()
  }

  pub fn local_ident_name(&self) -> &LocalIdentName {
    self.generator_options.local_ident_name()
  }

  pub fn exports_only(&self) -> bool {
    self.generator_options.exports_only()
  }

  pub fn named_exports(&self) -> bool {
    self.named_exports
  }

  pub fn es_module(&self) -> bool {
    self.generator_options.es_module()
  }

  pub fn resolve_import(&self) -> &CssParserImport {
    &self.resolve_import
  }

  pub fn export_type(&self) -> &Option<CssExportType> {
    self.generator_options.export_type()
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl ParserAndGenerator for CssParserAndGenerator {
  fn source_types(&self, module: &dyn Module, module_graph: &ModuleGraph) -> &[SourceType] {
    // 如果 export_type 是 style、css_style_sheet 或 text，我们只返回 JavaScript
    if matches!(
      self.export_type(),
      Some(CssExportType::Style) | Some(CssExportType::CssStyleSheet) | Some(CssExportType::Text)
    ) {
      return CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST;
    }

    if self.exports_only() {
      return CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST;
    }

    let no_need_js = module_graph
      .get_incoming_connections(&module.identifier())
      .all(|conn| {
        let dep = module_graph.dependency_by_id(&conn.dependency_id);
        matches!(
          dep.dependency_type(),
          DependencyType::CssImport | DependencyType::EsmImport
        )
      });

    if no_need_js {
      CSS_MODULE_SOURCE_TYPE_LIST
    } else {
      CSS_MODULE_AND_JS_SOURCE_TYPE_LIST
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
      module_match_resource,
      ..
    } = parse_context;

    build_info.strict = true;
    build_meta.exports_type = if self.named_exports() {
      BuildMetaExportsType::Namespace
    } else {
      BuildMetaExportsType::Default
    };
    build_meta.default_object = if self.named_exports() {
      BuildMetaDefaultObject::False
    } else {
      BuildMetaDefaultObject::Redirect
    };

    let source = remove_bom(source);
    let source_code = source.source().into_string_lossy();
    let resource_data = module_match_resource.unwrap_or(resource_data);
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
    let mut css_exports: Option<CssExports> = None;
    let mut css_local_names: Option<FxHashMap<String, String>> = None;

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
            )));
            continue;
          }
          // Check the import option
          let should_import = match self.resolve_import() {
            CssParserImport::Bool(b) => *b,
            CssParserImport::Func(f) => {
              // Call the filter function with the import arguments
              let args = CssParserImportContext {
                url: request.to_string(),
                media: media.map(|s| s.to_string()),
                resource_path: resource_path
                  .map(|p| p.as_str().to_string())
                  .unwrap_or_default(),
                supports: supports.map(|s| s.to_string()),
                layer: layer.map(|s| s.to_string()),
              };
              (f(args).await).unwrap_or(true)
            }
          };
          if !should_import {
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
          ))),
        css_module_lexer::Dependency::LocalClass { name, range, .. }
        | css_module_lexer::Dependency::LocalId { name, range, .. } => {
          let (_prefix, name) = name.split_at(1); // split '#' or '.'
          let name = unescape(name);

          let (local_ident, convention_names) = self
            .resolve_local_ident_and_update_exports(
              resource_data,
              compiler_options,
              &name,
              &mut css_exports,
            )
            .await?;

          let local_names = css_local_names.get_or_insert_default();
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
          let (local_ident, convention_names) = self
            .resolve_local_ident_and_update_exports(
              resource_data,
              compiler_options,
              &name,
              &mut css_exports,
            )
            .await?;
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
          let (local_ident, convention_names) = self
            .resolve_local_ident_and_update_exports(
              resource_data,
              compiler_options,
              &name,
              &mut css_exports,
            )
            .await?;

          let local_names = css_local_names.get_or_insert_default();
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
              names.clone(),
              vec![],
            )));
          }

          let convention = self.convention();
          let exports = css_exports.get_or_insert_default();
          for name in names {
            for local_class in local_classes.iter() {
              let convention_names = export_locals_convention(&name, convention);
              let convention_local_class = export_locals_convention(local_class, convention);

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
        }
        css_module_lexer::Dependency::ICSSExportValue { prop, value } => {
          let exports = css_exports.get_or_insert_default();
          let convention = self.convention();
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
                orig_name: prop.to_string(),
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

    build_info.css_exports = css_exports;
    build_info.css_local_names = css_local_names;

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
          .runtime_template
          .runtime_requirements_mut()
          .insert(RuntimeGlobals::HAS_CSS_MODULES);

        let mut source = ReplaceSource::new(source.clone());
        let compilation = generate_context.compilation;
        let mut init_fragments = ModuleInitFragments::default();
        let mut context = TemplateContext {
          compilation,
          module,
          runtime: generate_context.runtime,
          init_fragments: &mut init_fragments,
          concatenation_scope: generate_context.concatenation_scope.take(),
          data: generate_context.data,
          runtime_template: generate_context.runtime_template,
        };

        let module_graph = compilation.get_module_graph();
        module.get_dependencies().iter().for_each(|id| {
          let dep = module_graph.dependency_by_id(id);

          if let Some(dependency) = dep.as_dependency_code_generation() {
            if let Some(template) = dependency
              .dependency_template()
              .and_then(|template_type| compilation.get_dependency_template(template_type))
            {
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
          let dep = module_graph.dependency_by_id(&conn.dependency_id);

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
            if let Some(template) = dependency
              .dependency_template()
              .and_then(|dependency_type| compilation.get_dependency_template(dependency_type))
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

        generate_context.concatenation_scope = context.concatenation_scope.take();

        Ok(source.boxed())
      }
      SourceType::JavaScript => {
        let with_hmr = self.hot;

        if matches!(
          self.export_type(),
          Some(CssExportType::Style)
            | Some(CssExportType::CssStyleSheet)
            | Some(CssExportType::Text)
        ) {
          let mut concat_source = ConcatSource::default();

          // 处理 CSS 依赖并获取最终的 CSS 内容
          let mut source = ReplaceSource::new(source.clone());
          let compilation = generate_context.compilation;
          let mut init_fragments = ModuleInitFragments::default();
          let mut context = TemplateContext {
            compilation,
            module,
            runtime: generate_context.runtime,
            init_fragments: &mut init_fragments,
            concatenation_scope: generate_context.concatenation_scope.take(),
            data: generate_context.data,
            runtime_template: generate_context.runtime_template,
          };

          let module_graph = compilation.get_module_graph();
          module.get_dependencies().iter().for_each(|id| {
            let dep = module_graph.dependency_by_id(id);

            if let Some(dependency) = dep.as_dependency_code_generation() {
              if let Some(template) = dependency
                .dependency_template()
                .and_then(|template_type| compilation.get_dependency_template(template_type))
              {
                template.render(dependency, &mut source, &mut context)
              }
            }
          });

          generate_context.concatenation_scope = context.concatenation_scope.take();

          // 获取处理后的 CSS 文本
          let css_source = source.boxed();
          let css_text = css_source.source().into_string_lossy();

          // 转义 CSS 用于 JavaScript 字符串
          let css_js_string =
            serde_json::to_string(&css_text).map_err(|e| rspack_error::error!("{}", e))?;

          if matches!(self.export_type(), Some(CssExportType::Style)) {
            // 添加 CSS_INJECT_STYLE 运行时要求
            generate_context
              .runtime_template
              .runtime_requirements_mut()
              .insert(RuntimeGlobals::CSS_INJECT_STYLE);

            // 获取模块 ID
            let module_id = ChunkGraph::get_module_id(
              &generate_context.compilation.module_ids_artifact,
              module.identifier(),
            )
            .map(|id| id.to_string())
            .unwrap_or_default();

            // 生成注入调用
            let css_inject_call = format!(
              "{}({}, {});\n",
              generate_context
                .runtime_template
                .render_runtime_globals(&RuntimeGlobals::CSS_INJECT_STYLE),
              serde_json::to_string(&module_id).map_err(|e| rspack_error::error!("{}", e))?,
              css_js_string
            );
            concat_source.add(RawStringSource::from(css_inject_call));
          }

          // 生成导出代码，对于 CssStyleSheet、Text，我们直接在这里处理
          let exports_str = if matches!(self.export_type(), Some(CssExportType::CssStyleSheet)) {
            let build_info = module.build_info();
            let module_argument = generate_context
              .runtime_template
              .render_module_argument(ModuleArgument::Module);
            let (ns_obj, left, right) = if self.es_module() {
              (
                generate_context
                  .runtime_template
                  .render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
                "(".to_string(),
                ")".to_string(),
              )
            } else {
              (String::new(), String::new(), String::new())
            };

            let css_style_sheet_code = format!(
              "var __css_text = {};\n\
               var __css_style_sheet = new CSSStyleSheet();\n\
               __css_style_sheet.replaceSync(__css_text);\n",
              css_js_string
            );

            let exports_str = if let Some(exports) = &build_info.css_exports {
              if let Some(local_names) = &build_info.css_local_names {
                let unused_exports = crate::parser_and_generator::get_unused_local_ident(
                  exports,
                  local_names,
                  module.identifier(),
                  generate_context.runtime,
                  &generate_context.compilation.exports_info_artifact,
                );
                generate_context.data.insert(unused_exports);
              }

              let exports = crate::parser_and_generator::get_used_exports(
                exports,
                module.identifier(),
                generate_context.runtime,
                &generate_context.compilation.exports_info_artifact,
              );

              let (decl_name, exports_string) = crate::utils::stringified_exports(
                exports,
                compilation,
                generate_context.runtime_template,
                module,
                generate_context.runtime,
              )?;

              let hmr_code = if with_hmr {
                format!(
                  "// only invalidate when locals change\n\
                   var stringified_exports = JSON.stringify({decl_name});\n\
                   if ({module_argument}.hot.data && {module_argument}.hot.data.exports && {module_argument}.hot.data.exports != stringified_exports) {{\n\
                   {module_argument}.hot.invalidate();\n\
                   }} else {{\n\
                   {module_argument}.hot.accept();\n\
                   }}\n\
                   {module_argument}.hot.dispose(function(data) {{ data.exports = stringified_exports; }});"
                )
              } else {
                String::new()
              };

              let mut code = format!(
                "{css_style_sheet_code}{exports_string}\n{hmr_code}\n{ns_obj}{left}{module_argument}.exports = Object.assign(__css_style_sheet, {decl_name})",
              );
              code += &right;
              code += ";\n";
              code
            } else {
              format!(
                "{css_style_sheet_code}{ns_obj}{left}{module_argument}.exports = __css_style_sheet{right};\n{}",
                if with_hmr {
                  format!("{module_argument}.hot.accept();\n")
                } else {
                  String::new()
                }
              )
            };

            RawStringSource::from(exports_str).boxed()
          } else if matches!(self.export_type(), Some(CssExportType::Text)) {
            let build_info = module.build_info();
            let module_argument = generate_context
              .runtime_template
              .render_module_argument(ModuleArgument::Module);
            let (ns_obj, left, right) = if self.es_module() {
              (
                generate_context
                  .runtime_template
                  .render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
                "(".to_string(),
                ")".to_string(),
              )
            } else {
              (String::new(), String::new(), String::new())
            };

            let exports_str = if let Some(exports) = &build_info.css_exports {
              if let Some(local_names) = &build_info.css_local_names {
                let unused_exports = crate::parser_and_generator::get_unused_local_ident(
                  exports,
                  local_names,
                  module.identifier(),
                  generate_context.runtime,
                  &generate_context.compilation.exports_info_artifact,
                );
                generate_context.data.insert(unused_exports);
              }

              let exports = crate::parser_and_generator::get_used_exports(
                exports,
                module.identifier(),
                generate_context.runtime,
                &generate_context.compilation.exports_info_artifact,
              );

              let (decl_name, exports_string) = crate::utils::stringified_exports(
                exports,
                compilation,
                generate_context.runtime_template,
                module,
                generate_context.runtime,
              )?;

              let hmr_code = if with_hmr {
                format!(
                  "// only invalidate when locals change\n\
                   var stringified_exports = JSON.stringify({decl_name});\n\
                   if ({module_argument}.hot.data && {module_argument}.hot.data.exports && {module_argument}.hot.data.exports != stringified_exports) {{\n\
                   {module_argument}.hot.invalidate();\n\
                   }} else {{\n\
                   {module_argument}.hot.accept();\n\
                   }}\n\
                   {module_argument}.hot.dispose(function(data) {{ data.exports = stringified_exports; }});"
                )
              } else {
                String::new()
              };

              // 使用不同的方式构建字符串，避免 format! 中的花括号冲突
              let mut code = String::new();
              code.push_str(&exports_string);
              code.push_str("\n");
              code.push_str(&hmr_code);
              code.push_str("\n");
              code.push_str(&ns_obj);
              code.push_str(&left);
              code.push_str(&module_argument);
              code.push_str(".exports = Object.assign({}, ");
              code.push_str(&decl_name);
              code.push_str(")");
              code.push_str(&right);
              code.push_str(";\n");
              // 添加 default 属性
              code.push_str(&module_argument);
              code.push_str(".default = ");
              code.push_str(&css_js_string);
              code.push_str(";\n");
              code
            } else {
              format!(
                "{ns_obj}{left}{module_argument}.exports = {css_js_string}{right};\n{}",
                if with_hmr {
                  format!("{module_argument}.hot.accept();\n")
                } else {
                  String::new()
                }
              )
            };

            RawStringSource::from(exports_str).boxed()
          } else {
            self.generate_js_exports(module, generate_context, with_hmr)?
          };
          concat_source.add(exports_str);
          Ok(concat_source.boxed())
        } else {
          self.generate_js_exports(module, generate_context, with_hmr)
        }
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
    if self.exports_only() {
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
    self.es_module().dyn_hash(&mut hasher);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

impl CssParserAndGenerator {
  async fn resolve_local_ident_and_update_exports(
    &self,
    resource_data: &ResourceData,
    compiler_options: &CompilerOptions,
    name: &str,
    css_exports: &mut Option<CssExports>,
  ) -> Result<(String, Vec<String>)> {
    let local_ident = LocalIdentOptions::new(
      resource_data,
      self.local_ident_name(),
      compiler_options,
      self.generator_options.local_ident_hash_digest.as_ref(),
      self.generator_options.local_ident_hash_digest_length,
      self.generator_options.local_ident_hash_function.as_ref(),
      self.generator_options.local_ident_hash_salt.as_ref(),
    )
    .get_local_ident(name)
    .await?;
    let convention = self.convention();
    let exports = css_exports.get_or_insert_default();
    let convention_names = export_locals_convention(name, convention);
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

  fn generate_js_exports(
    &self,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
    with_hmr: bool,
  ) -> Result<BoxSource> {
    let build_info = module.build_info();
    if generate_context.concatenation_scope.is_some() {
      let mut concate_source = ConcatSource::default();
      if let Some(ref exports) = build_info.css_exports {
        let exports_info_artifact = &generate_context.compilation.exports_info_artifact;
        if let Some(local_names) = &build_info.css_local_names {
          let unused_exports = get_unused_local_ident(
            exports,
            local_names,
            module.identifier(),
            generate_context.runtime,
            exports_info_artifact,
          );
          generate_context.data.insert(unused_exports);
        }
        let exports = get_used_exports(
          exports,
          module.identifier(),
          generate_context.runtime,
          exports_info_artifact,
        );

        css_modules_exports_to_concatenate_module_string(
          exports,
          module,
          generate_context,
          &mut concate_source,
        )?;
      }
      Ok(concate_source.boxed())
    } else {
      let exports_info = generate_context
        .compilation
        .exports_info_artifact
        .get_exports_info_data(&module.identifier());
      let (ns_obj, left, right) = if self.es_module()
        && exports_info
          .other_exports_info()
          .get_used(generate_context.runtime)
          != UsageState::Unused
      {
        (
          generate_context
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
          "(".to_string(),
          ")".to_string(),
        )
      } else {
        (String::new(), String::new(), String::new())
      };
      let exports_str = if let Some(exports) = &build_info.css_exports {
        if let Some(local_names) = &build_info.css_local_names {
          let unused_exports = get_unused_local_ident(
            exports,
            local_names,
            module.identifier(),
            generate_context.runtime,
            &generate_context.compilation.exports_info_artifact,
          );
          generate_context.data.insert(unused_exports);
        }

        let exports = get_used_exports(
          exports,
          module.identifier(),
          generate_context.runtime,
          &generate_context.compilation.exports_info_artifact,
        );

        css_modules_exports_to_string(
          exports,
          module,
          generate_context.compilation,
          generate_context.runtime,
          generate_context.runtime_template,
          &ns_obj,
          &left,
          &right,
          with_hmr,
        )?
      } else {
        let module_argument = generate_context
          .runtime_template
          .render_module_argument(ModuleArgument::Module);
        format!(
          "{}{}{module_argument}.exports = {{}}{};\n{}",
          &ns_obj,
          &left,
          &right,
          if with_hmr {
            format!("{module_argument}.hot.accept();\n")
          } else {
            Default::default()
          }
        )
      };
      Ok(RawStringSource::from(exports_str).boxed())
    }
  }
}

pub fn get_used_exports<'a>(
  exports: &'a CssExports,
  identifier: ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
  exports_info_artifact: &ExportsInfoArtifact,
) -> FxIndexMap<&'a str, &'a FxIndexSet<CssExport>> {
  let exports_info = exports_info_artifact
    .get_exports_info_optional(&identifier)
    .map(|info| info.as_data(exports_info_artifact));

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

pub fn get_unused_local_ident(
  exports: &CssExports,
  local_names: &FxHashMap<String, String>,
  identifier: ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
  exports_info_artifact: &ExportsInfoArtifact,
) -> CodeGenerationDataUnusedLocalIdent {
  let exports_names = exports.iter().fold(
    FxHashMap::<&str, FxHashSet<Atom>>::default(),
    |mut map, (name, css_exports)| {
      css_exports.iter().for_each(|css_export| {
        if let Some(set) = map.get_mut(css_export.orig_name.as_str()) {
          set.insert(Atom::from(name.clone()));
        } else {
          map.insert(
            &css_export.orig_name,
            FxHashSet::from_iter([Atom::from(name.clone())]),
          );
        }
      });
      map
    },
  );

  let exports_info = exports_info_artifact
    .get_exports_info_optional(&identifier)
    .map(|info| info.as_data(exports_info_artifact));

  CodeGenerationDataUnusedLocalIdent {
    idents: exports_names
      .iter()
      .filter(|(_, export_names)| {
        export_names.iter().all(|export_name| {
          let export_info = exports_info
            .as_ref()
            .map(|info| info.get_read_only_export_info(export_name));

          if let Some(export_info) = export_info {
            matches!(export_info.get_used(runtime), UsageState::Unused)
          } else {
            false
          }
        })
      })
      .filter_map(|(css_name, _)| local_names.get(*css_name).cloned())
      .collect(),
  }
}

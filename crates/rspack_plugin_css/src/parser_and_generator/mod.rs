#![allow(clippy::comparison_chain)]

use std::sync::Arc;

use indexmap::IndexMap;
use preset_env_base::query::{Query, Targets};
use rspack_core::{
  ast::css::Ast as CssAst,
  rspack_sources::{
    MapOptions, RawSource, Source, SourceExt, SourceMap, SourceMapSource, SourceMapSourceOptions,
  },
  BuildMetaExportsType, GenerateContext, GenerationResult, Module, ModuleType, ParseContext,
  ParseResult, ParserAndGenerator, SourceType,
};
use rspack_core::{AstOrSource, ModuleAst, ModuleDependency};
use rspack_error::{
  internal_error, Diagnostic, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray,
};
use rustc_hash::FxHashSet;
use sugar_path::SugarPath;
use swc_core::css::visit::VisitMutWithPath;
use swc_core::{
  css::{
    modules::CssClassName,
    parser::parser::ParserConfig,
    prefixer::{options::Options, prefixer},
    visit::VisitMutWith,
  },
  ecma::atoms::JsWord,
};

use crate::dependency::{
  collect_dependency_code_generation_visitors, CssComposeDependency,
  DependencyCodeGenerationVisitors, DependencyVisitor,
};
use crate::plugin::CssConfig;
use crate::swc_css_compiler::{SwcCssSourceMapGenConfig, SWC_COMPILER};
use crate::utils::{css_modules_exports_to_string, ModulesTransformConfig};
use crate::{pxtorem::px_to_rem::px_to_rem, visitors::analyze_dependencies};

pub(crate) static CSS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::JavaScript, SourceType::Css];

pub(crate) static CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST: &[SourceType; 1] =
  &[SourceType::JavaScript];

/// Compat for @rspack/postcss-loader css modules
#[derive(serde::Deserialize)]
struct RspackPostcssModules {
  rspack_postcss_modules: String,
}

#[derive(Debug)]
pub struct CssParserAndGenerator {
  pub config: CssConfig,
  pub meta: Option<String>,
  pub exports: Option<IndexMap<JsWord, Vec<CssClassName>>>,
}

impl CssParserAndGenerator {
  pub fn get_query(&self) -> Option<Query> {
    // TODO(h-a-n-a): figure out if the prefixer visitMut is stateless
    // I need to clone the preset_env every time, due to I don't know if it is stateless
    // If it is true, I reduce this clone
    if !self.config.targets.is_empty() {
      Some(Query::Multiple(self.config.targets.clone()))
    } else {
      None
    }
  }
}

impl ParserAndGenerator for CssParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    if self.config.modules.exports_only {
      CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST
    } else {
      CSS_MODULE_SOURCE_TYPE_LIST
    }
  }

  fn size(&self, module: &dyn Module, source_type: &SourceType) -> f64 {
    match source_type {
      SourceType::JavaScript => {
        // meta + `module.exports = ...`
        self
          .meta
          .as_ref()
          .map(|item| item.len() as f64 + 17.0)
          .unwrap_or(0.0)
      }
      SourceType::Css => module.original_source().map_or(0, |source| source.size()) as f64,
      _ => unreachable!(),
    }
  }

  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      additional_data,
      module_type,
      resource_data,
      compiler_options,
      build_info,
      build_meta,
      code_generation_dependencies,
      ..
    } = parse_context;
    build_info.strict = true;
    build_meta.exports_type = BuildMetaExportsType::Namespace;
    let cm: Arc<swc_core::common::SourceMap> = Default::default();
    let content = source.source().to_string();
    let css_modules = matches!(module_type, ModuleType::CssModule);
    let TWithDiagnosticArray {
      inner: mut stylesheet,
      mut diagnostic,
    } = SWC_COMPILER.parse_file(
      cm.clone(),
      &parse_context.resource_data.resource_path.to_string_lossy(),
      content,
      ParserConfig {
        css_modules,
        legacy_ie: true,
        ..Default::default()
      },
    )?;

    if let Some(query) = self.get_query() {
      stylesheet.visit_mut_with(&mut prefixer(Options {
        env: Some(Targets::Query(query)),
      }));
    }

    if let Some(config) = self.config.postcss.pxtorem.clone() {
      stylesheet.visit_mut_with(&mut px_to_rem(config));
    }

    let locals = if css_modules {
      let filename = &resource_data
        .resource_path
        .relative(&compiler_options.context);
      let result = swc_core::css::modules::compile(
        &mut stylesheet,
        ModulesTransformConfig {
          filename,
          local_name_ident: &self.config.modules.local_ident_name,
        },
      );
      let mut exports: IndexMap<JsWord, _> = result.renamed.into_iter().collect();
      exports.sort_keys();
      Some(exports)
    } else {
      None
    };

    let mut dependencies = analyze_dependencies(
      &mut stylesheet,
      code_generation_dependencies,
      &mut diagnostic,
    );

    let  dependencies = if let Some(locals) = &locals && !locals.is_empty() {
      let mut dep_set = FxHashSet::default();
      let compose_deps = locals.iter().flat_map(|(_, value)| value).filter_map(|name| if let CssClassName::Import { from, .. } = name {
        if dep_set.contains(from.as_ref()) {
          None
        } else {
          dep_set.insert(from.to_string());
          Some(Box::new(CssComposeDependency::new(from.to_string(), None)) as Box<dyn ModuleDependency>)
        }
      } else {
        None
      });
      dependencies.extend(compose_deps);
      dependencies
    } else {
      dependencies
    };

    self.meta = additional_data.and_then(|data| if data.is_empty() { None } else { Some(data) });
    self.exports = locals;

    if self.exports.is_some() && let Some(meta) = &self.meta && serde_json::from_str::<RspackPostcssModules>(meta).is_ok() {
      diagnostic.push(Diagnostic::warn("CSS Modules".to_string(), format!("file: {} is using `postcss.modules` and `builtins.css.modules` to process css modules at the same time, rspack will use `builtins.css.modules`'s result.", resource_data.resource_path.display()), 0, 0));
    }

    Ok(
      ParseResult {
        dependencies,
        presentational_dependencies: vec![],
        ast_or_source: AstOrSource::Ast(ModuleAst::Css(CssAst::new(stylesheet, cm))),
      }
      .with_diagnostic(diagnostic),
    )
  }

  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    ast_or_source: &rspack_core::AstOrSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<rspack_core::GenerationResult> {
    let result = match generate_context.requested_source_type {
      SourceType::Css => {
        let devtool = &generate_context.compilation.options.devtool;
        let mut ast = ast_or_source
          .to_owned()
          .try_into_ast()
          .expect("Expected AST for CSS generator, please file an issue.")
          .try_into_css()
          .expect("Expected CSS AST for CSS generation, please file an issue.");
        let dependency_visitors =
          collect_dependency_code_generation_visitors(module, generate_context)?;
        let cm = ast.get_context().source_map.clone();
        let stylesheet = ast.get_root_mut();
        let DependencyCodeGenerationVisitors {
          visitors,
          root_visitors,
          ..
        } = dependency_visitors;

        {
          if !visitors.is_empty() {
            stylesheet.visit_mut_with_path(
              &mut DependencyVisitor::new(
                visitors
                  .iter()
                  .map(|(ast_path, visitor)| (ast_path, &**visitor))
                  .collect(),
              ),
              &mut Default::default(),
            );
          }

          for (_, root_visitor) in root_visitors {
            stylesheet.visit_mut_with(&mut root_visitor.create());
          }
        }

        let (code, source_map) = SWC_COMPILER.codegen(
          cm,
          stylesheet,
          SwcCssSourceMapGenConfig {
            enable: devtool.source_map(),
            inline_sources_content: !devtool.no_sources(),
            emit_columns: !devtool.cheap(),
          },
        )?;
        if let Some(source_map) = source_map {
          let source = SourceMapSource::new(SourceMapSourceOptions {
            value: code,
            name: module.try_as_normal_module()?.user_request().to_string(),
            source_map: SourceMap::from_slice(&source_map)
              .map_err(|e| internal_error!(e.to_string()))?,
            // Safety: original source exists in code generation
            original_source: Some(
              module
                .original_source()
                .expect("Failed to get original source, please file an issue.")
                .source()
                .to_string(),
            ),
            // Safety: original source exists in code generation
            inner_source_map: module
              .original_source()
              .expect("Failed to get original source, please file an issue.")
              .map(&MapOptions::default()),
            remove_original_source: false,
          })
          .boxed();
          Ok(source)
        } else {
          Ok(RawSource::from(code).boxed())
        }
      }
      SourceType::JavaScript => {
        let locals = if let Some(exports) = &self.exports {
          css_modules_exports_to_string(
            exports,
            module,
            generate_context.compilation,
            &self.config.modules.locals_convention,
          )?
        } else if let Some(meta) = &self.meta
          && let Ok(meta) = serde_json::from_str::<RspackPostcssModules>(meta)
        {
          format!("module.exports = {};\n", meta.rspack_postcss_modules)
        } else if generate_context.compilation.options.dev_server.hot  {
          "module.hot.accept();".to_string()
        } else {
          "".to_string()
        };
        Ok(RawSource::from(locals).boxed())
      }
      _ => Err(internal_error!(
        "Unsupported source type: {:?}",
        generate_context.requested_source_type
      )),
    }?;

    Ok(GenerationResult {
      ast_or_source: result.into(),
    })
  }
}

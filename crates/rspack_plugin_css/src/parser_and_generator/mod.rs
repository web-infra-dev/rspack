#![allow(clippy::comparison_chain)]

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use preset_env_base::query::{Query, Targets};
use regex::Regex;
use rspack_core::{
  rspack_sources::{
    MapOptions, RawSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource,
    SourceMapSourceOptions,
  },
  AstOrSource, BuildMetaExportsType, GenerateContext, GenerationResult, Module, ModuleType,
  ParseContext, ParseResult, ParserAndGenerator, SourceType, TemplateContext,
};
use rspack_core::{ModuleDependency, RuntimeGlobals};
use rspack_error::{
  internal_error, Diagnostic, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray,
};
use rustc_hash::FxHashSet;
use sugar_path::SugarPath;
use swc_core::{
  css::{
    modules::CssClassName,
    parser::parser::ParserConfig,
    prefixer::{options::Options, prefixer},
    visit::VisitMutWith,
  },
  ecma::atoms::JsWord,
};

use crate::plugin::CssConfig;
use crate::swc_css_compiler::SwcCssSourceMapGenConfig;
use crate::utils::{css_modules_exports_to_string, ModulesTransformConfig};
use crate::{dependency::CssComposeDependency, swc_css_compiler::SwcCssCompiler};
use crate::{pxtorem::px_to_rem::px_to_rem, visitors::analyze_dependencies};

static REGEX_IS_MODULES: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.module(s)?\.[^.]+$").expect("Invalid regex"));

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
      module_user_request,
      resource_data,
      compiler_options,
      build_info,
      build_meta,
      code_generation_dependencies,
      ..
    } = parse_context;

    build_info.strict = true;
    build_meta.exports_type = BuildMetaExportsType::Default;

    let swc_compiler = SwcCssCompiler::default();

    let source_code = source.source().into_owned();
    let resource_path = &parse_context.resource_data.resource_path;

    let is_enable_css_modules = match module_type {
      ModuleType::CssModule => true,
      ModuleType::CssAuto
        if REGEX_IS_MODULES.is_match(resource_path.to_string_lossy().as_ref()) =>
      {
        true
      }
      _ => false,
    };

    let TWithDiagnosticArray {
      inner: mut stylesheet,
      mut diagnostic,
    } = swc_compiler.parse_file(
      &resource_path.to_string_lossy(),
      source_code,
      ParserConfig {
        css_modules: is_enable_css_modules,
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

    let locals = if is_enable_css_modules {
      let result = swc_core::css::modules::compile(
        &mut stylesheet,
        ModulesTransformConfig::new(
          &resource_data
            .resource_path
            .relative(&compiler_options.context),
          &self.config.modules.local_ident_name,
          &compiler_options.output,
        ),
      );
      let mut exports: IndexMap<JsWord, _> = result.renamed.into_iter().collect();
      exports.sort_keys();
      Some(exports)
    } else {
      None
    };
    let devtool = &compiler_options.devtool;

    let (code, source_map) = swc_compiler.codegen(
      &stylesheet,
      SwcCssSourceMapGenConfig {
        enable: devtool.source_map(),
        inline_sources_content: !devtool.no_sources(),
        emit_columns: !devtool.cheap(),
      },
    )?;

    let TWithDiagnosticArray {
      inner: new_stylesheet_ast,
      diagnostic: new_diagnostic,
    } = SwcCssCompiler::default().parse_file(
      &parse_context.resource_data.resource_path.to_string_lossy(),
      code.clone(),
      Default::default(),
    )?;
    diagnostic.extend(new_diagnostic);

    let mut dependencies = analyze_dependencies(
      &new_stylesheet_ast,
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

    let new_source = if let Some(source_map) = source_map {
      SourceMapSource::new(SourceMapSourceOptions {
        value: code,
        name: module_user_request,
        source_map: SourceMap::from_slice(&source_map)
          .map_err(|e| internal_error!(e.to_string()))?,
        // Safety: original source exists in code generation
        original_source: Some(source.source().to_string()),
        // Safety: original source exists in code generation
        inner_source_map: source.map(&MapOptions::default()),
        remove_original_source: false,
      })
      .boxed()
    } else {
      RawSource::from(code).boxed()
    };

    Ok(
      ParseResult {
        dependencies,
        presentational_dependencies: vec![],
        ast_or_source: AstOrSource::new(None, Some(new_source)),
        analyze_result: Default::default(),
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
        let mut source = ReplaceSource::new(ast_or_source.to_owned().try_into_source()?);
        let compilation = generate_context.compilation;
        let mut context = TemplateContext {
          compilation,
          module,
          runtime_requirements: generate_context.runtime_requirements,
          init_fragments: &mut vec![],
        };

        let mgm = compilation
          .module_graph
          .module_graph_module_by_identifier(&module.identifier())
          .expect("should have module graph module");

        mgm.dependencies.iter().for_each(|id| {
          if let Some(dependency) = compilation
            .module_graph
            .dependency_by_id(id)
            .expect("should have dependency")
            .as_code_generatable_dependency()
          {
            dependency.apply(&mut source, &mut context)
          }
        });

        if let Some(dependencies) = module.get_presentational_dependencies() {
          dependencies
            .iter()
            .for_each(|dependency| dependency.apply(&mut source, &mut context));
        };

        Ok(source.boxed())
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
        generate_context
          .runtime_requirements
          .insert(RuntimeGlobals::MODULE);
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

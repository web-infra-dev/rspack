#![allow(clippy::comparison_chain)]

use std::collections::HashMap;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use rkyv::{from_bytes, to_bytes, AlignedVec};
use rspack_core::{
  rspack_sources::{
    BoxSource, MapOptions, RawSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource,
    SourceMapSourceOptions,
  },
  BoxDependency, BuildExtraDataType, BuildMetaExportsType, GenerateContext, Module, ModuleType,
  ParseContext, ParseResult, ParserAndGenerator, SourceType, TemplateContext,
};
use rspack_core::{ModuleInitFragments, RuntimeGlobals};
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rustc_hash::FxHashSet;
use sugar_path::SugarPath;
use swc_core::{css::parser::parser::ParserConfig, ecma::atoms::JsWord};

use crate::{
  dependency::CssComposeDependency,
  swc_css_compiler::{SwcCssCompiler, SwcCssSourceMapGenConfig},
};
use crate::{
  plugin::CssConfig,
  utils::{css_modules_exports_to_string, ModulesTransformConfig},
};
use crate::{
  utils::{stringify_css_modules_exports_elements, stringify_css_modules_exports_key},
  visitors::analyze_dependencies,
};

static REGEX_IS_MODULES: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.module(s)?\.[^.]+$").expect("Invalid regex"));

pub(crate) static CSS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::JavaScript, SourceType::Css];

pub(crate) static CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST: &[SourceType; 1] =
  &[SourceType::JavaScript];

type CssExportsType = Option<IndexMap<Vec<String>, Vec<(String, Option<String>)>>>;

#[derive(Debug)]
pub struct CssParserAndGenerator {
  pub config: CssConfig,
  pub exports: CssExportsType,
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
      SourceType::JavaScript => 42.0,
      SourceType::Css => module.original_source().map_or(0, |source| source.size()) as f64,
      _ => unreachable!(),
    }
  }

  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
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

    let mut source_code = source.source().into_owned();
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

    let devtool = &compiler_options.devtool;
    let mut source_map = None;
    let mut diagnostic_vec = vec![];

    if is_enable_css_modules {
      let TWithDiagnosticArray {
        inner: mut stylesheet,
        diagnostic,
      } = swc_compiler.parse_file(
        &resource_path.to_string_lossy(),
        source_code,
        ParserConfig {
          css_modules: is_enable_css_modules,
          legacy_ie: true,
          ..Default::default()
        },
      )?;

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

      self.exports = Some(IndexMap::from_iter(
        exports
          .iter()
          .map(|(key, elements)| {
            (
              stringify_css_modules_exports_key(key, &self.config.modules.locals_convention),
              stringify_css_modules_exports_elements(elements),
            )
          })
          .collect::<Vec<_>>(),
      ));

      let (code, map) = swc_compiler.codegen(
        &stylesheet,
        SwcCssSourceMapGenConfig {
          enable: devtool.source_map(),
          inline_sources_content: !devtool.no_sources(),
          emit_columns: !devtool.cheap(),
        },
      )?;
      source_code = code;
      source_map = map;
      diagnostic_vec.extend(diagnostic)
    }

    let TWithDiagnosticArray {
      inner: new_stylesheet_ast,
      diagnostic: new_diagnostic,
    } = SwcCssCompiler::default().parse_file(
      &parse_context.resource_data.resource_path.to_string_lossy(),
      source_code.clone(),
      Default::default(),
    )?;
    diagnostic_vec.extend(new_diagnostic);

    let mut dependencies = analyze_dependencies(
      &new_stylesheet_ast,
      code_generation_dependencies,
      &mut diagnostic_vec,
    );

    let dependencies = if let Some(locals) = &self.exports
      && !locals.is_empty()
    {
      let mut dep_set = FxHashSet::default();
      let compose_deps = locals
        .iter()
        .flat_map(|(_, value)| value)
        .filter_map(|(_, from)| {
          if let Some(from) = from {
            if dep_set.contains(&from) {
              None
            } else {
              dep_set.insert(from);
              Some(Box::new(CssComposeDependency::new(from.to_owned(), None)) as BoxDependency)
            }
          } else {
            None
          }
        });
      dependencies.extend(compose_deps);
      dependencies
    } else {
      dependencies
    };

    let new_source = if devtool.source_map() {
      if let Some(source_map) = source_map {
        SourceMapSource::new(SourceMapSourceOptions {
          value: source_code,
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
        source
      }
    } else {
      RawSource::from(source_code).boxed()
    };

    Ok(
      ParseResult {
        dependencies,
        blocks: vec![],
        presentational_dependencies: vec![],
        source: new_source,
        analyze_result: Default::default(),
      }
      .with_diagnostic(diagnostic_vec),
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
        let mut source = ReplaceSource::new(source.clone());
        let compilation = generate_context.compilation;
        let mut init_fragments = ModuleInitFragments::default();
        let mut context = TemplateContext {
          compilation,
          module,
          runtime_requirements: generate_context.runtime_requirements,
          runtime: generate_context.runtime,
          init_fragments: &mut init_fragments,
        };

        let mgm = compilation
          .module_graph
          .module_graph_module_by_identifier(&module.identifier())
          .expect("should have module graph module");

        mgm.all_dependencies.iter().for_each(|id| {
          if let Some(dependency) = compilation
            .module_graph
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

        Ok(source.boxed())
      }
      SourceType::JavaScript => {
        let locals = if let Some(exports) = &self.exports {
          css_modules_exports_to_string(exports, module, generate_context.compilation)?
        } else if generate_context.compilation.options.dev_server.hot {
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

    Ok(result)
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
      let data = from_bytes::<Option<IndexMap<Vec<String>, Vec<(String, Option<String>)>>>>(data)
        .expect("Failed to resume extra data");
      self.exports = data;
    }
  }
}

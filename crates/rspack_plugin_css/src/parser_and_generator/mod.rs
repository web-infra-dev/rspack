#![allow(clippy::comparison_chain)]

use std::collections::HashMap;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use rkyv::{from_bytes, to_bytes, AlignedVec};
use rspack_core::{
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics,
  rspack_sources::{
    BoxSource, ConcatSource, MapOptions, RawSource, ReplaceSource, Source, SourceExt, SourceMap,
    SourceMapSource, SourceMapSourceOptions,
  },
  BoxDependency, BuildExtraDataType, BuildMetaDefaultObject, BuildMetaExportsType, ChunkGraph,
  CssExportsConvention, ErrorSpan, GenerateContext, LocalIdentName, Module, ModuleGraph,
  ModuleType, ParseContext, ParseResult, ParserAndGenerator, SourceType, TemplateContext,
};
use rspack_core::{ModuleInitFragments, RuntimeGlobals};
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rustc_hash::FxHashSet;
use swc_core::{
  css::{parser::parser::ParserConfig, visit::VisitWith},
  ecma::atoms::Atom,
};

use crate::{
  dependency::{CssComposeDependency, CssModuleExportDependency},
  swc_css_compiler::{SwcCssCompiler, SwcCssSourceMapGenConfig},
  utils::css_modules_exports_to_concatenate_module_string,
};
use crate::{
  utils::{css_modules_exports_to_string, ModulesTransformConfig},
  visitors::ExportsAnalyzer,
};
use crate::{
  utils::{export_locals_convention, stringify_css_modules_exports_elements},
  visitors::analyze_dependencies,
};

static REGEX_IS_MODULES: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.module(s)?\.[^.]+$").expect("Invalid regex"));

pub(crate) static CSS_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::JavaScript, SourceType::Css];

pub(crate) static CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST: &[SourceType; 1] =
  &[SourceType::JavaScript];

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct CssExport(pub String, pub ErrorSpan, pub Option<String>);

pub type CssExportsType = IndexMap<Vec<String>, Vec<CssExport>>;

#[derive(Debug)]
pub struct CssParserAndGenerator {
  pub convention: CssExportsConvention,
  pub local_ident_name: Option<LocalIdentName>,
  pub exports_only: bool,
  pub named_exports: bool,
  pub exports: Option<CssExportsType>,
  pub es_module: bool,
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
      module_user_request,
      module_source_map_kind,
      resource_data,
      compiler_options,
      build_info,
      build_meta,
      code_generation_dependencies,
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

    let mut source_map = None;
    let mut diagnostic_vec = vec![];

    let mut exports_pairs = vec![];
    let mut presentational_dependencies = None;
    let mut exports = if is_enable_css_modules {
      let mut stylesheet = swc_compiler.parse_file(
        &resource_path.to_string_lossy(),
        source_code.clone(),
        ParserConfig {
          css_modules: is_enable_css_modules,
          legacy_ie: true,
          ..Default::default()
        },
      )?;

      let result = swc_core::css::modules::compile(
        &mut stylesheet,
        ModulesTransformConfig::new(
          resource_data,
          self
            .local_ident_name
            .as_ref()
            .expect("should have local_ident_name for module_type css/auto or css/module"),
          compiler_options,
        ),
      );
      let exports: IndexMap<Atom, _> = result.renamed.into_iter().collect();

      let (code, map) = swc_compiler.codegen(
        &stylesheet,
        SwcCssSourceMapGenConfig {
          enable: module_source_map_kind.enabled(),
          inline_sources_content: module_source_map_kind.source_map(),
          emit_columns: !module_source_map_kind.cheap(),
        },
      )?;
      source_code = code;
      source_map = map;
      Some(exports)
    } else {
      None
    };

    let new_stylesheet_ast = SwcCssCompiler::default().parse_file(
      &parse_context.resource_data.resource_path.to_string_lossy(),
      source_code.clone(),
      Default::default(),
    )?;

    if let Some(exports) = &mut exports {
      let mut exports_analyzer = ExportsAnalyzer::new(&source_code);
      new_stylesheet_ast.visit_with(&mut exports_analyzer);
      presentational_dependencies = Some(exports_analyzer.presentation_deps);

      for (key, value) in exports_analyzer.exports {
        exports.insert(key, vec![value]);
      }

      let normalized_exports = IndexMap::from_iter(
        exports
          .iter()
          .map(|(name, elements)| {
            let mut names = export_locals_convention(name, &self.convention);
            names.sort_unstable();
            names.dedup();
            (names, stringify_css_modules_exports_elements(elements))
          })
          .collect::<Vec<_>>(),
      );

      for (k, v) in normalized_exports.iter() {
        for kk in k {
          exports_pairs.push((kk.to_string(), v[0].0.to_owned()));
        }
      }

      self.exports = Some(normalized_exports);
    }

    let mut dependencies = analyze_dependencies(
      &new_stylesheet_ast,
      code_generation_dependencies,
      &mut diagnostic_vec,
      &source_code,
      module_user_request,
    );
    for (k, v) in exports_pairs {
      dependencies.push(Box::new(CssModuleExportDependency::new(
        k[1..k.len() - 1].to_owned(),
        v,
      )));
    }

    let dependencies = if let Some(locals) = &self.exports
      && !locals.is_empty()
    {
      let mut dep_set = FxHashSet::default();
      let mut compose_deps = locals
        .iter()
        .flat_map(|(_, value)| value)
        .filter_map(|CssExport(_, span, from)| {
          if let Some(from) = from {
            if dep_set.contains(&from) {
              None
            } else {
              dep_set.insert(from);
              Some(Box::new(CssComposeDependency::new(from.to_owned(), *span)) as BoxDependency)
            }
          } else {
            None
          }
        })
        .collect::<Vec<_>>();

      compose_deps.sort_by(|a, b| match (a.span(), b.span()) {
        (Some(span_a), Some(span_b)) => span_a.cmp(&span_b),
        _ => unreachable!(),
      });

      dependencies.extend(compose_deps);
      dependencies
    } else {
      dependencies
    };

    let new_source = if module_source_map_kind.enabled() {
      if let Some(source_map) = source_map {
        SourceMapSource::new(SourceMapSourceOptions {
          value: source_code,
          name: module_user_request,
          source_map: SourceMap::from_slice(&source_map)
            .expect("should be able to generate source-map"),
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
        presentational_dependencies: presentational_dependencies.unwrap_or_default(),
        source: new_source,
        analyze_result: Default::default(),
        side_effects_bailout: None,
      }
      .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
        diagnostic_vec,
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
      let data = from_bytes::<Option<CssExportsType>>(data).expect("Failed to resume extra data");
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

pub mod generator;
mod parser;

use std::{borrow::Cow, sync::LazyLock};

use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  ChunkGraph, Compilation, CssModuleGeneratorOptions, CssModuleParserOptions, DependencyType,
  ExportsInfoArtifact, GenerateContext, Module, ModuleGraph, ModuleIdentifier, ModuleInitFragments,
  NormalModule, ParseContext, ParseResult, ParserAndGenerator, RuntimeGlobals, RuntimeSpec,
  SourceType, TemplateContext, UsageState,
  rspack_sources::{BoxSource, ReplaceSource, Source, SourceExt},
};
pub use rspack_core::{CssExport, CssExports};
use rspack_error::{Result, TWithDiagnosticArray};
use rspack_hash::{RspackHash, RspackHashDigest};
use rspack_util::{
  atom::Atom,
  ext::DynHash,
  fx_hash::{FxIndexMap, FxIndexSet},
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  dependency::{CssImportDependency, CssMedia, CssSupports},
  parser_and_generator::{generator::CssModuleGenerator, parser::CssModuleParser},
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

pub type CssExportsRef<'a> = FxIndexMap<&'a str, &'a FxIndexSet<CssExport>>;

#[cacheable]
#[derive(Debug)]
pub struct CssParserAndGenerator {
  pub generator_options: CssModuleGeneratorOptions,
  pub parser_options: CssModuleParserOptions,
  pub exports_only: bool,
  pub hot: bool,
}

impl CssParserAndGenerator {
  pub fn new(
    generator_options: CssModuleGeneratorOptions,
    parser_options: CssModuleParserOptions,
  ) -> Self {
    let exports_only = generator_options
      .exports_only
      .expect("should have exports_only");

    Self {
      generator_options,
      parser_options,
      exports_only,
      hot: false,
    }
  }

  pub fn es_module(&self) -> bool {
    self
      .generator_options
      .es_module
      .expect("should have es_module")
  }
}

pub fn get_used_exports<'a>(
  exports: &'a CssExports,
  identifier: ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
  exports_info_artifact: &ExportsInfoArtifact,
) -> CssExportsRef<'a> {
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

static REGEX_CUSTOM_PROPERTY_IDENT: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"(^|[^-_a-zA-Z0-9])--([_a-zA-Z][-_a-zA-Z0-9]*)").expect("Invalid regex")
});

#[cacheable_dyn]
#[async_trait::async_trait]
impl ParserAndGenerator for CssParserAndGenerator {
  fn source_types(&self, module: &dyn Module, module_graph: &ModuleGraph) -> &[SourceType] {
    if self.exports_only {
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
    CssModuleParser::new(
      &self.generator_options,
      &self.parser_options,
      self.exports_only,
      parse_context,
    )
    .parse()
    .await
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
        CssModuleGenerator::new(module, generate_context, self.hot, self.es_module())
          .generate_javascript_source()
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
    self.es_module().dyn_hash(&mut hasher);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

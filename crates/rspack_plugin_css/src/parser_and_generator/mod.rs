pub mod generator;
mod parser;
mod source_builder;

use std::{borrow::Cow, sync::LazyLock};

use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, ChunkGraph, Compilation, CssBuildInfo,
  CssModuleGeneratorOptions, CssModuleParserOptions, DependencyType, ExportsInfoArtifact,
  GenerateContext, GeneratorOptions, Module, ModuleGraph, ModuleIdentifier, ModuleInitFragments,
  NormalModule, ParseContext, ParseResult, ParserAndGenerator, ParserOptions, RuntimeGlobals,
  RuntimeSpec, SourceType, TemplateContext, UsageState,
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
use smol_str::SmolStr;
pub(crate) use source_builder::CssSourceBuilder;

use crate::parser_and_generator::{generator::CssModuleGenerator, parser::CssModuleParser};

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
#[derive(Debug, Default)]
pub struct CssParserAndGenerator {
  pub hot: bool,
}

impl CssParserAndGenerator {
  pub fn new() -> Self {
    Self::default()
  }

  fn exports_only(generator_options: &CssModuleGeneratorOptions) -> bool {
    generator_options
      .exports_only
      .expect("should have exports_only")
  }

  fn es_module(generator_options: &CssModuleGeneratorOptions) -> bool {
    generator_options.es_module.expect("should have es_module")
  }
}

fn css_generator_options(
  generator_options: Option<&GeneratorOptions>,
) -> &CssModuleGeneratorOptions {
  generator_options
    .and_then(GeneratorOptions::get_css_module)
    .expect("should have CssModuleGeneratorOptions")
}

fn css_parser_options(parser_options: Option<&ParserOptions>) -> &CssModuleParserOptions {
  parser_options
    .and_then(ParserOptions::get_css_module)
    .expect("should have CssModuleParserOptions")
}

pub fn get_used_exports<'a>(
  css_build_info: &'a CssBuildInfo,
  identifier: ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<CssExportsRef<'a>> {
  let exports = css_build_info.exports()?;
  let exports_info = exports_info_artifact
    .get_exports_info_optional(&identifier)
    .map(|info| info.as_data(exports_info_artifact));

  Some(
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
      .collect(),
  )
}

#[derive(Debug, Clone)]
pub struct CodeGenerationDataUnusedLocalIdent {
  pub(crate) idents: FxHashSet<SmolStr>,
}

pub fn get_unused_local_ident(
  css_build_info: &CssBuildInfo,
  identifier: ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<CodeGenerationDataUnusedLocalIdent> {
  let exports = css_build_info.exports()?;
  let local_names = css_build_info.local_names()?;
  let exports_names = exports.iter().fold(
    FxHashMap::<&str, FxHashSet<Atom>>::default(),
    |mut map, (name, css_exports)| {
      css_exports.iter().for_each(|css_export| {
        if let Some(set) = map.get_mut(css_export.orig_name.as_str()) {
          set.insert(Atom::from(name.as_str()));
        } else {
          map.insert(
            css_export.orig_name.as_str(),
            FxHashSet::from_iter([Atom::from(name.as_str())]),
          );
        }
      });
      map
    },
  );

  let exports_info = exports_info_artifact
    .get_exports_info_optional(&identifier)
    .map(|info| info.as_data(exports_info_artifact));

  Some(CodeGenerationDataUnusedLocalIdent {
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
  })
}

static REGEX_CUSTOM_PROPERTY_IDENT: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"(^|[^-_a-zA-Z0-9])--([_a-zA-Z][-_a-zA-Z0-9]*)").expect("Invalid regex")
});

#[cacheable_dyn]
#[async_trait::async_trait]
impl ParserAndGenerator for CssParserAndGenerator {
  fn source_types(&self, module: &dyn Module, module_graph: &ModuleGraph) -> &[SourceType] {
    let generator_options = css_generator_options(
      module
        .as_normal_module()
        .expect("CssParserAndGenerator should only be used by NormalModule")
        .get_generator_options(),
    );
    if Self::exports_only(generator_options) {
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
    let generator_options = css_generator_options(parse_context.module_generator_options);
    let parser_options = css_parser_options(parse_context.module_parser_options);
    let named_exports = parser_options
      .named_exports
      .expect("should have named_exports");

    {
      let build_info = &mut *parse_context.build_info;
      let build_meta = &mut *parse_context.build_meta;

      build_info.strict = true;
      build_meta.exports_type = if named_exports {
        BuildMetaExportsType::Namespace
      } else {
        BuildMetaExportsType::Default
      };
      build_meta.default_object = if named_exports {
        BuildMetaDefaultObject::False
      } else {
        BuildMetaDefaultObject::Redirect
      };
    }

    let exports_only = Self::exports_only(generator_options);

    CssModuleParser::new(
      generator_options,
      parser_options,
      exports_only,
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
        let es_module = Self::es_module(css_generator_options(
          generate_context.module_generator_options,
        ));
        CssModuleGenerator::new(module, generate_context, self.hot, es_module)
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
    module: &dyn rspack_core::Module,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    let generator_options = css_generator_options(
      module
        .as_normal_module()
        .expect("CssParserAndGenerator should only be used by NormalModule")
        .get_generator_options(),
    );
    if Self::exports_only(generator_options) {
      None
    } else {
      // CSS Module cannot be concatenated as it must appear in css chunk, if it's
      // concatenated, it will be removed from module graph
      Some("Module Concatenation is not implemented for CssParserAndGenerator".into())
    }
  }

  async fn get_runtime_hash(
    &self,
    module: &NormalModule,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) -> Result<RspackHashDigest> {
    let mut hasher = RspackHash::from(&compilation.options.output);
    Self::es_module(css_generator_options(module.get_generator_options())).dyn_hash(&mut hasher);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

pub mod generator;
mod parser;
mod source_builder;

use std::{
  borrow::Cow,
  sync::{Arc, LazyLock},
};

use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, ChunkGraph, Compilation,
  CssAutoOrModuleParserOptions, CssBuildInfo, CssExportType, DependencyType, ExportsInfoArtifact,
  GenerateContext, Module, ModuleGraph, ModuleIdentifier, NormalModule, ParseContext, ParseResult,
  ParserAndGenerator, ParserOptions, ResolvedModuleOptions, RuntimeSpec, SourceType, UsageState,
  rspack_sources::{BoxSource, Source},
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

use crate::{
  parser_and_generator::{generator::CssModuleGenerator, parser::CssModuleParser},
  utils::css_generator_options,
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
#[derive(Debug, Default)]
pub struct CssParserAndGenerator {
  pub hot: bool,
  pub export_type: Option<CssExportType>,
  pub exports_only: bool,
  pub es_module: bool,
}

impl CssParserAndGenerator {
  pub fn new(module_options: Arc<ResolvedModuleOptions>) -> Self {
    let export_type = module_options
      .parser_options()
      .and_then(|options| options.get_css_module())
      .and_then(|options| options.export_type);
    let generator_options = css_generator_options(module_options.generator_options());
    let exports_only = generator_options
      .exports_only
      .expect("should have exports_only");
    let es_module = generator_options.es_module.expect("should have es_module");

    Self {
      export_type,
      exports_only,
      es_module,
      ..Default::default()
    }
  }

  fn effective_export_type(&self, module: &dyn Module) -> Option<CssExportType> {
    module
      .build_info()
      .css
      .as_deref()
      .and_then(|css| css.export_type)
      .or(self.export_type)
  }
}

fn is_css_module(module_type: &rspack_core::ModuleType, resource_path: Option<&str>) -> bool {
  match module_type {
    rspack_core::ModuleType::CssModule => true,
    rspack_core::ModuleType::CssAuto => {
      resource_path.is_some_and(|path| REGEX_IS_MODULES.is_match(path))
    }
    _ => false,
  }
}

fn css_parser_options(parser_options: Option<&ParserOptions>) -> &CssAutoOrModuleParserOptions {
  parser_options
    .and_then(ParserOptions::get_css_module)
    .expect("CssParserOptions should be normalized to CssAutoOrModule")
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
    let export_type = self.effective_export_type(module);
    if matches!(
      export_type,
      Some(CssExportType::Style | CssExportType::CssStyleSheet | CssExportType::Text)
    ) {
      return CSS_MODULE_EXPORTS_ONLY_SOURCE_TYPE_LIST;
    }

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
    let generator_options = css_generator_options(parse_context.module_generator_options);
    let parser_options = css_parser_options(parse_context.module_parser_options);
    let named_exports = parser_options
      .named_exports
      .expect("should have named_exports");

    {
      let build_info = &mut *parse_context.build_info;
      let build_meta = &mut *parse_context.build_meta;

      build_info.strict = true;
      build_meta.is_css_module = is_css_module(
        parse_context.module_type,
        parse_context.resource_data.path().map(|path| path.as_str()),
      );
      build_meta.need_id_in_concatenation = self.export_type == Some(CssExportType::Style);
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

    let exports_only = generator_options
      .exports_only
      .expect("should have exports_only");

    CssModuleParser::new(
      generator_options,
      parser_options,
      exports_only,
      parse_context,
    )
    .parse()
    .await
  }

  async fn generate(
    &self,
    source: &BoxSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    match generate_context.requested_source_type {
      SourceType::Css => Ok(
        CssModuleGenerator::new(
          source.clone(),
          module,
          generate_context,
          self.hot,
          self.es_module,
        )
        .generate_css_source(),
      ),
      SourceType::JavaScript => CssModuleGenerator::new(
        source.clone(),
        module,
        generate_context,
        self.hot,
        self.es_module,
      )
      .generate_javascript_source(),
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
    if !self.es_module {
      Some("Module Concatenation is not implemented for CommonJS css exports".into())
    } else if self.effective_export_type(module) == Some(CssExportType::Style)
      && module
        .build_info()
        .css
        .as_deref()
        .is_some_and(|css_build_info| css_build_info.has_render_conditions())
    {
      Some("Module Concatenation is not implemented for conditional css style exports".into())
    } else if self.exports_only || self.effective_export_type(module).is_some() {
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
    self.es_module.dyn_hash(&mut hasher);
    self.exports_only.dyn_hash(&mut hasher);
    self.effective_export_type(module).dyn_hash(&mut hasher);
    Ok(hasher.digest(&compilation.options.output.hash_digest))
  }
}

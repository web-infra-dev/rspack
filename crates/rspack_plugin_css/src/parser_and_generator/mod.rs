pub mod generator;
pub mod impl_parser_and_generator;

use std::sync::LazyLock;

use regex::Regex;
use rspack_cacheable::cacheable;
use rspack_core::{
  CompilerOptions, CssExportsConvention, CssModuleGeneratorOptions, CssModuleParserOptions,
  CssParserImport, Dependency, ExportsInfoArtifact, LocalIdentName, ModuleIdentifier, ResourceData,
  RuntimeSpec, SourceType, UsageState,
};
pub use rspack_core::{CssExport, CssExports};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxIndexMap, FxIndexSet},
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  dependency::{
    CssLocalIdentDependency, CssSelfReferenceLocalIdentDependency,
    CssSelfReferenceLocalIdentReplacement,
  },
  parser_and_generator::generator::update_css_exports,
  utils::{LocalIdentOptions, export_locals_convention, unescape},
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
  pub hot: bool,
}

impl CssParserAndGenerator {
  pub fn new(
    generator_options: CssModuleGeneratorOptions,
    parser_options: CssModuleParserOptions,
  ) -> Self {
    Self {
      generator_options,
      parser_options,
      hot: false,
    }
  }

  pub fn convention(&self) -> &CssExportsConvention {
    self
      .generator_options
      .exports_convention
      .as_ref()
      .expect("should have convention for module_type css/auto, css/global or css/module")
  }

  pub fn local_ident_name(&self) -> &LocalIdentName {
    self
      .generator_options
      .local_ident_name
      .as_ref()
      .expect("should have local_ident_name for module_type css/auto, css/global or css/module")
  }

  pub fn exports_only(&self) -> bool {
    self
      .generator_options
      .exports_only
      .expect("should have exports_only")
  }

  pub fn named_exports(&self) -> bool {
    self
      .parser_options
      .named_exports
      .expect("should have named_exports")
  }

  pub fn es_module(&self) -> bool {
    self
      .generator_options
      .es_module
      .expect("should have es_module")
  }

  pub fn resolve_import(&self) -> &CssParserImport {
    self
      .parser_options
      .resolve_import
      .as_ref()
      .unwrap_or(&CssParserImport::Bool(true))
  }

  pub fn url(&self) -> bool {
    self.parser_options.url.expect("should have url")
  }

  pub async fn handle_local_ident_usage(
    &self,
    name: &str,
    range: css_module_lexer::Range,
    resource_data: &ResourceData,
    compiler_options: &CompilerOptions,
    css_exports: &mut Option<CssExports>,
    dependencies: &mut Vec<Box<dyn Dependency>>,
  ) -> rspack_error::Result<()> {
    let name = unescape(name);
    let (local_ident, convention_names) = self
      .resolve_local_ident_and_update_exports(resource_data, compiler_options, &name, css_exports)
      .await?;
    dependencies.push(Box::new(CssSelfReferenceLocalIdentDependency::new(
      convention_names,
      vec![CssSelfReferenceLocalIdentReplacement {
        local_ident,
        range: (range.start, range.end).into(),
      }],
    )));
    Ok(())
  }

  pub async fn handle_local_ident_declaration(
    &self,
    name: &str,
    range: css_module_lexer::Range,
    resource_data: &ResourceData,
    compiler_options: &CompilerOptions,
    css_exports: &mut Option<CssExports>,
    css_local_names: &mut Option<FxHashMap<String, String>>,
    dependencies: &mut Vec<Box<dyn Dependency>>,
  ) -> rspack_error::Result<()> {
    let name = unescape(name);
    let (local_ident, convention_names) = self
      .resolve_local_ident_and_update_exports(resource_data, compiler_options, &name, css_exports)
      .await?;

    let local_names = css_local_names.get_or_insert_default();
    local_names.insert(name.into_owned(), local_ident.clone());

    dependencies.push(Box::new(CssLocalIdentDependency::new(
      local_ident,
      convention_names,
      range.start,
      range.end,
    )));
    Ok(())
  }

  pub async fn resolve_local_ident_and_update_exports(
    &self,
    resource_data: &ResourceData,
    compiler_options: &CompilerOptions,
    name: &str,
    css_exports: &mut Option<CssExports>,
  ) -> rspack_error::Result<(String, Vec<String>)> {
    let local_ident =
      LocalIdentOptions::new(resource_data, self.local_ident_name(), compiler_options)
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

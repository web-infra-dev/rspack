#![allow(clippy::unwrap_used)]

mod legacy_case;
mod template;
mod visit;

use std::fmt::Debug;

use cow_utils::CowUtils;
use heck::{ToKebabCase, ToLowerCamelCase, ToSnakeCase};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::Deserialize;
use swc_core::{
  atoms::Wtf8Atom,
  common::{BytePos, DUMMY_SP, Span, SyntaxContext, errors::HANDLER, util::take::Take},
  ecma::{
    ast::{
      Ident, ImportDecl, ImportDefaultSpecifier, ImportNamedSpecifier, ImportPhase,
      ImportSpecifier, Module, ModuleDecl, ModuleExportName, ModuleItem, Str,
    },
    atoms::Atom,
    visit::{VisitMut, VisitWith, visit_mut_pass},
  },
};

use crate::{
  legacy_case::{identifier_to_legacy_kebab_case, identifier_to_legacy_snake_case},
  template::{Template, TemplateEngine},
  visit::IdentComponent,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RawStyleConfig {
  pub style_library_directory: Option<String>,
  pub custom: Option<String>,
  pub css: Option<String>,
  pub bool: Option<bool>,
}

impl From<RawStyleConfig> for StyleConfig {
  fn from(raw_style_config: RawStyleConfig) -> Self {
    if let Some(style_library_directory) = raw_style_config.style_library_directory {
      Self::StyleLibraryDirectory(style_library_directory)
    } else if let Some(custom) = raw_style_config.custom {
      Self::Custom(CustomTransform::Tpl(custom))
    } else if raw_style_config.css.is_some() {
      Self::Css
    } else if let Some(bool) = raw_style_config.bool {
      Self::Bool(bool)
    } else {
      Self::None
    }
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RawImportOptions {
  pub library_name: String,
  pub library_directory: Option<String>, // default to `lib`
  pub custom_name: Option<String>,
  pub custom_style_name: Option<String>, // If this is set, `style` option will be ignored
  pub style: Option<RawStyleConfig>,
  pub camel_to_dash_component_name: Option<bool>, // default to true
  pub transform_to_default_import: Option<bool>,
  pub ignore_es_component: Option<Vec<String>>,
  pub ignore_style_component: Option<Vec<String>>,
}

impl From<RawImportOptions> for ImportOptions {
  fn from(plugin_import: RawImportOptions) -> Self {
    let RawImportOptions {
      library_name,
      library_directory,
      custom_name,
      custom_style_name,
      style,
      camel_to_dash_component_name,
      transform_to_default_import,
      ignore_es_component,
      ignore_style_component,
    } = plugin_import;

    Self {
      library_name,
      library_directory,
      custom_name: custom_name.map(CustomTransform::Tpl),
      custom_style_name: custom_style_name.map(CustomTransform::Tpl),
      style: style.map(Into::into),
      camel_to_dash_component_name,
      transform_to_default_import,
      ignore_es_component,
      ignore_style_component,
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
pub enum StyleConfig {
  StyleLibraryDirectory(String),
  #[serde(skip)]
  Custom(CustomTransform),
  Css,
  Bool(bool),
  None,
}

#[derive(Deserialize)]
pub enum CustomTransform {
  #[serde(skip)]
  Fn(Box<dyn Sync + Send + Fn(String) -> Option<String>>),
  Tpl(String),
}

impl Clone for CustomTransform {
  fn clone(&self) -> Self {
    match self {
      Self::Fn(_) => panic!("Function cannot be cloned"),
      Self::Tpl(s) => Self::Tpl(s.clone()),
    }
  }
}

impl Debug for CustomTransform {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CustomTransform::Fn(_) => f.write_str("Function"),
      CustomTransform::Tpl(t) => f.write_str(t),
    }
  }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ImportOptions {
  pub library_name: String,
  pub library_directory: Option<String>, // default to `lib`
  #[serde(skip)]
  pub custom_name: Option<CustomTransform>,
  #[serde(skip)]
  pub custom_style_name: Option<CustomTransform>, // If this is set, `style` option will be ignored
  pub style: Option<StyleConfig>,

  pub camel_to_dash_component_name: Option<bool>, // default to true
  pub transform_to_default_import: Option<bool>,

  pub ignore_es_component: Option<Vec<String>>,
  pub ignore_style_component: Option<Vec<String>>,
}

const CUSTOM_JS: &str = "CUSTOM_JS_NAME";
const CUSTOM_STYLE: &str = "CUSTOM_STYLE";
const CUSTOM_STYLE_NAME: &str = "CUSTOM_STYLE_NAME";

/// Panic:
///
/// Panics in sometimes if [swc_core::common::errors::HANDLER] is not provided.
pub fn plugin_import(
  config: &Vec<ImportOptions>,
) -> swc_core::ecma::visit::VisitMutPass<ImportPlugin<'_>> {
  let mut renderer = TemplateEngine::new();

  renderer.register_helper("kebabCase", |value| value.to_kebab_case());
  renderer.register_helper("legacyKebabCase", |value| {
    identifier_to_legacy_kebab_case(value)
  });
  renderer.register_helper("camelCase", |value| value.to_lower_camel_case());
  renderer.register_helper("snakeCase", |value| value.to_snake_case());
  renderer.register_helper("legacySnakeCase", |value| {
    identifier_to_legacy_snake_case(value)
  });
  renderer.register_helper("upperCase", |value| {
    value.cow_to_ascii_uppercase().into_owned()
  });
  renderer.register_helper("lowerCase", |value| {
    value.cow_to_ascii_lowercase().into_owned()
  });

  config.iter().enumerate().for_each(|(index, item)| {
    if let Some(CustomTransform::Tpl(tpl)) = &item.custom_name {
      match Template::parse(tpl) {
        Ok(template) => {
          renderer.register_template(format!("{}{}", item.library_name, CUSTOM_JS), template);
        }
        Err(e) => {
          HANDLER.with(|handler| {
            handler.err(&format!(
              "[builtin:swc-loader] Failed to parse option \"rspackExperiments.import[{}].customName\".\nReason: {}",
              index,
              &e.to_string()
            ))
          });
        }
      }
    }

    if let Some(CustomTransform::Tpl(tpl)) = &item.custom_style_name {
      match Template::parse(tpl) {
        Ok(template) => {
          renderer
            .register_template(format!("{}{}", item.library_name, CUSTOM_STYLE_NAME), template);
        }
        Err(e) => {
          HANDLER.with(|handler| {
            handler.err(&format!(
              "[builtin:swc-loader] Failed to parse option \"rspackExperiments.import[{}].customStyleName\".\nReason: {}",
              index,
              &e.to_string()
            ))
          });
        }
      }
    }

    if let Some(StyleConfig::Custom(CustomTransform::Tpl(tpl))) = &item.style {
      match Template::parse(tpl) {
        Ok(template) => {
          renderer.register_template(format!("{}{}", item.library_name, CUSTOM_STYLE), template);
        }
        Err(e) => {
          HANDLER.with(|handler| {
            handler.err(&format!(
              "[builtin:swc-loader] Failed to parse option \"rspackExperiments.import[{}].style\".\nReason: {}",
              index,
              &e.to_string()
            ))
          });
        }
      }
    }
  });

  visit_mut_pass(ImportPlugin { config, renderer })
}

#[derive(Debug)]
struct EsSpec {
  source: String,
  default_spec: String,
  as_name: Option<String>,
  use_default_import: bool,
  mark: u32,
}

pub struct ImportPlugin<'a> {
  pub config: &'a Vec<ImportOptions>,
  pub renderer: TemplateEngine<'a>,
}

impl ImportPlugin<'_> {
  // return (import_es, import_css)
  fn transform(&self, name: String, config: &ImportOptions) -> (Option<String>, Option<String>) {
    let should_ignore = &config
      .ignore_es_component
      .as_ref()
      .is_some_and(|list| list.iter().any(|c| c == &name));

    if *should_ignore {
      return (None, None);
    }

    let should_ignore_css = &config
      .ignore_style_component
      .as_ref()
      .is_some_and(|list| list.iter().any(|c| c == &name));

    let transformed_name = if config.camel_to_dash_component_name.unwrap_or(true) {
      name.to_kebab_case()
    } else {
      name.clone()
    };

    let path = if let Some(transform) = &config.custom_name {
      match transform {
        CustomTransform::Fn(f) => Ok(f(name.clone())),
        CustomTransform::Tpl(_) => self
          .renderer
          .render(
            format!("{}{}", &config.library_name, CUSTOM_JS).as_str(),
            &render_context(name.clone()),
          )
          .map(Some),
      }
    } else {
      Ok(Some(format!(
        "{}/{}/{}",
        &config.library_name,
        config.library_directory.as_deref().unwrap_or("lib"),
        transformed_name
      )))
    };

    let path = path.unwrap_or_else(|err| {
      HANDLER.with(|handler| {
        handler.err(&format!(
          "[builtin:swc-loader] Failed to parse option \
       \"rspackExperiments.import[i].customName\".\nReason: {err}"
        ));
      });
      None
    });

    let Some(js_source) = path else {
      return (None, None);
    };

    let css = if *should_ignore_css {
      None
    } else if let Some(custom) = &config.custom_style_name {
      match custom {
        CustomTransform::Fn(f) => f(name),
        CustomTransform::Tpl(_) => self
          .renderer
          .render(
            &format!("{}{}", &config.library_name, CUSTOM_STYLE_NAME),
            &render_context(name),
          )
          .map_or_else(
            |err| {
              HANDLER.with(|handler| {
                handler.err(&format!(
                  "[builtin:swc-loader] Failed to parse option \
                  \"rspackExperiments.import[i].customStyleName\".\nReason: {err}"
                ));
              });
              None
            },
            Some,
          ),
      }
    } else if let Some(style) = &config.style {
      match style {
        StyleConfig::StyleLibraryDirectory(lib) => Some(format!(
          "{}/{}/{}",
          config.library_name, lib, &transformed_name
        )),
        StyleConfig::Custom(custom) => match custom {
          CustomTransform::Fn(f) => f(js_source.clone()),
          CustomTransform::Tpl(_) => self
            .renderer
            .render(
              &format!("{}{}", config.library_name, CUSTOM_STYLE),
              &render_context(js_source.clone()),
            )
            .map_or_else(
              |err| {
                HANDLER.with(|handler| {
                  handler.err(&format!(
                    "[builtin:swc-loader] Failed to parse option \
                     \"rspackExperiments.import[i].style\".\nReason: {err}"
                  ));
                });
                None
              },
              Some,
            ),
        },
        StyleConfig::Css => Some(format!("{js_source}/style/css")),
        StyleConfig::Bool(should_transform) => {
          if *should_transform {
            Some(format!("{}/style", &js_source))
          } else {
            None
          }
        }
        StyleConfig::None => None,
      }
    } else {
      None
    };

    (Some(js_source), css)
  }
}

impl VisitMut for ImportPlugin<'_> {
  fn visit_mut_module(&mut self, module: &mut Module) {
    // use visitor to collect all ident reference, and then remove imported component and type that is never referenced
    let mut visitor = IdentComponent {
      ident_set: HashSet::default(),
      type_ident_set: HashSet::default(),
      in_ts_type_ref: false,
    };
    module.body.visit_with(&mut visitor);

    let ident_referenced = |ident: &Ident| -> bool { visitor.ident_set.contains(&ident.to_id()) };
    let type_ident_referenced =
      |ident: &Ident| -> bool { visitor.type_ident_set.contains(&ident.to_id()) };

    let mut specifiers_css = vec![];
    let mut specifiers_es = vec![];
    let mut specifiers_rm_es = HashSet::default();

    let config = &self.config;

    for (item_index, item) in module.body.iter_mut().enumerate() {
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(var)) = item {
        let source = &*var.src.value;

        if let Some(child_config) = config
          .iter()
          .find(|&c| c.library_name == source.to_string_lossy())
        {
          let mut rm_specifier = HashSet::default();

          for (specifier_idx, specifier) in var.specifiers.iter().enumerate() {
            match specifier {
              ImportSpecifier::Named(s) => {
                let imported = s.imported.as_ref().map(|imported| match imported {
                  ModuleExportName::Ident(ident) => ident.sym.to_string(),
                  ModuleExportName::Str(str) => str.value.to_string_lossy().to_string(),
                });

                let as_name: Option<String> = imported.is_some().then(|| s.local.sym.to_string());
                let ident: String = imported.unwrap_or_else(|| s.local.sym.to_string());

                let mark = s.local.ctxt.as_u32();

                if ident_referenced(&s.local) {
                  let use_default_import = child_config.transform_to_default_import.unwrap_or(true);

                  let (import_es_source, import_css_source) =
                    self.transform(ident.clone(), child_config);

                  if let Some(source) = import_es_source {
                    specifiers_es.push(EsSpec {
                      source,
                      default_spec: ident,
                      as_name,
                      use_default_import,
                      mark,
                    });
                    rm_specifier.insert(specifier_idx);
                  }

                  if let Some(source) = import_css_source {
                    specifiers_css.push(source);
                  }
                } else if type_ident_referenced(&s.local) {
                  // type referenced
                } else {
                  // not referenced, should tree shaking
                  rm_specifier.insert(specifier_idx);
                }
              }
              ImportSpecifier::Default(_s) => {}
              ImportSpecifier::Namespace(_ns) => {}
            }
          }
          if rm_specifier.len() == var.specifiers.len() {
            // all specifier remove, just remove whole stmt
            specifiers_rm_es.insert(item_index);
          } else {
            // only remove some specifier
            var.specifiers = var
              .specifiers
              .take()
              .into_iter()
              .enumerate()
              .filter_map(|(idx, spec)| (!rm_specifier.contains(&idx)).then_some(spec))
              .collect();
          }
        }
      }
    }

    module.body = module
      .body
      .take()
      .into_iter()
      .enumerate()
      .filter_map(|(idx, stmt)| (!specifiers_rm_es.contains(&idx)).then_some(stmt))
      .collect();

    let body = &mut module.body;

    for js_source in specifiers_es {
      let js_source_ref = js_source.source.as_str();
      let dec = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        specifiers: if js_source.use_default_import {
          vec![ImportSpecifier::Default(ImportDefaultSpecifier {
            span: DUMMY_SP,
            local: Ident {
              ctxt: SyntaxContext::from_u32(js_source.mark),
              span: Span::new(BytePos::DUMMY, BytePos::DUMMY),
              sym: Atom::from(js_source.as_name.unwrap_or(js_source.default_spec).as_str()),
              optional: false,
            },
          })]
        } else {
          vec![ImportSpecifier::Named(ImportNamedSpecifier {
            span: DUMMY_SP,
            imported: if js_source.as_name.is_some() {
              Some(ModuleExportName::Ident(Ident {
                span: DUMMY_SP,
                ctxt: Default::default(),
                sym: Atom::from(js_source.default_spec.as_str()),
                optional: false,
              }))
            } else {
              None
            },
            local: Ident {
              ctxt: SyntaxContext::from_u32(js_source.mark),
              span: Span::new(BytePos::DUMMY, BytePos::DUMMY),
              sym: Atom::from(js_source.as_name.unwrap_or(js_source.default_spec).as_str()),
              optional: false,
            },
            is_type_only: false,
          })]
        },
        src: Box::new(Str {
          span: DUMMY_SP,
          value: Wtf8Atom::from(js_source_ref),
          raw: None,
        }),
        type_only: false,
        with: Default::default(),
        phase: ImportPhase::default(),
      }));
      body.insert(0, dec);
    }

    for css_source in specifiers_css {
      let dec = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        specifiers: vec![],
        src: Box::new(Str {
          span: DUMMY_SP,
          value: Wtf8Atom::from(css_source),
          raw: None,
        }),
        type_only: false,
        with: Default::default(),
        phase: ImportPhase::default(),
      }));
      body.insert(0, dec);
    }
  }
}

fn render_context(s: String) -> HashMap<&'static str, String> {
  let mut ctx = HashMap::default();
  ctx.insert("member", s);
  ctx
}

#![allow(clippy::unwrap_used)]

mod visit;
use std::fmt::Debug;

use handlebars::{Context, Helper, HelperResult, Output, RenderContext, Template};
use heck::{ToKebabCase, ToLowerCamelCase, ToSnakeCase};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::Deserialize;
use swc_core::{
  common::{errors::HANDLER, util::take::Take, BytePos, Span, SyntaxContext, DUMMY_SP},
  ecma::{
    ast::{
      Ident, ImportDecl, ImportDefaultSpecifier, ImportNamedSpecifier, ImportPhase,
      ImportSpecifier, Module, ModuleDecl, ModuleExportName, ModuleItem, Str,
    },
    atoms::Atom,
    visit::{as_folder, Fold, VisitMut, VisitWith},
  },
};

use crate::visit::IdentComponent;

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
pub struct PluginImportConfig {
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
pub fn plugin_import(config: &Vec<PluginImportConfig>) -> impl Fold + '_ {
  let mut renderer = handlebars::Handlebars::new();

  renderer.register_helper(
    "kebabCase",
    Box::new(
      |helper: &Helper<'_>,
       _: &'_ handlebars::Handlebars<'_>,
       _: &'_ Context,
       _: &mut RenderContext<'_, '_>,
       out: &mut dyn Output|
       -> HelperResult {
        let param = helper
          .param(0)
          .and_then(|v| v.value().as_str())
          .unwrap_or("");
        out.write(param.to_kebab_case().as_ref())?;
        Ok(())
      },
    ),
  );

  renderer.register_helper(
    "camelCase",
    Box::new(
      |helper: &Helper<'_>,
       _: &'_ handlebars::Handlebars<'_>,
       _: &'_ Context,
       _: &mut RenderContext<'_, '_>,
       out: &mut dyn Output|
       -> HelperResult {
        let param = helper
          .param(0)
          .and_then(|v| v.value().as_str())
          .unwrap_or("");
        out.write(param.to_lower_camel_case().as_ref())?;
        Ok(())
      },
    ),
  );

  renderer.register_helper(
    "snakeCase",
    Box::new(
      |helper: &Helper<'_>,
       _: &'_ handlebars::Handlebars<'_>,
       _: &'_ Context,
       _: &mut RenderContext<'_, '_>,
       out: &mut dyn Output|
       -> HelperResult {
        let param = helper
          .param(0)
          .and_then(|v| v.value().as_str())
          .unwrap_or("");
        out.write(param.to_snake_case().as_ref())?;
        Ok(())
      },
    ),
  );

  renderer.register_helper(
    "upperCase",
    Box::new(
      |helper: &Helper<'_>,
       _: &'_ handlebars::Handlebars<'_>,
       _: &'_ Context,
       _: &mut RenderContext<'_, '_>,
       out: &mut dyn Output|
       -> HelperResult {
        let param = helper
          .param(0)
          .and_then(|v| v.value().as_str())
          .unwrap_or("");
        out.write(param.to_uppercase().as_ref())?;
        Ok(())
      },
    ),
  );

  renderer.register_helper(
    "lowerCase",
    Box::new(
      |helper: &Helper<'_>,
       _: &'_ handlebars::Handlebars<'_>,
       _: &'_ Context,
       _: &mut RenderContext<'_, '_>,
       out: &mut dyn Output|
       -> HelperResult {
        let param = helper
          .param(0)
          .and_then(|v| v.value().as_str())
          .unwrap_or("");
        out.write(param.to_lowercase().as_ref())?;
        Ok(())
      },
    ),
  );

  config.iter().for_each(|cfg| {
    if let Some(CustomTransform::Tpl(tpl)) = &cfg.custom_name {
      renderer.register_template(
        &(cfg.library_name.clone() + CUSTOM_JS),
        Template::compile(tpl).unwrap(),
      )
    }

    if let Some(CustomTransform::Tpl(tpl)) = &cfg.custom_style_name {
      renderer.register_template(
        &(cfg.library_name.clone() + CUSTOM_STYLE_NAME),
        Template::compile(tpl).unwrap(),
      )
    }

    if let Some(StyleConfig::Custom(CustomTransform::Tpl(tpl))) = &cfg.style {
      renderer.register_template(
        &(cfg.library_name.clone() + CUSTOM_STYLE),
        Template::compile(tpl).unwrap(),
      )
    }
  });

  as_folder(ImportPlugin { config, renderer })
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
  pub config: &'a Vec<PluginImportConfig>,
  pub renderer: handlebars::Handlebars<'a>,
}

impl<'a> ImportPlugin<'a> {
  // return (import_es, import_css)
  fn transform(
    &self,
    name: String,
    config: &PluginImportConfig,
  ) -> (Option<String>, Option<String>) {
    let should_ignore = &config
      .ignore_es_component
      .as_ref()
      .map(|list| list.iter().any(|c| c == &name))
      .unwrap_or(false);

    if *should_ignore {
      return (None, None);
    }

    let should_ignore_css = &config
      .ignore_style_component
      .as_ref()
      .map(|list| list.iter().any(|c| c == &name))
      .unwrap_or(false);

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
        config
          .library_directory
          .as_ref()
          .unwrap_or(&"lib".to_string()),
        transformed_name
      )))
    };

    let path = match path {
      Ok(p) => p,
      Err(e) => {
        HANDLER.with(|handler| {
          handler.err(&e.to_string());
        });
        None
      }
    };

    if path.is_none() {
      return (None, None);
    }

    let js_source = path.unwrap();

    let css = if *should_ignore_css {
      None
    } else if let Some(custom) = &config.custom_style_name {
      match custom {
        CustomTransform::Fn(f) => f(name),
        CustomTransform::Tpl(_) => Some(
          self
            .renderer
            .render(
              &format!("{}{}", &config.library_name, CUSTOM_STYLE_NAME),
              &render_context(name),
            )
            .unwrap(),
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
          CustomTransform::Tpl(_) => Some(
            self
              .renderer
              .render(
                &format!("{}{}", config.library_name, CUSTOM_STYLE),
                &render_context(js_source.clone()),
              )
              .expect("Should success"),
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

impl<'a> VisitMut for ImportPlugin<'a> {
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

        if let Some(child_config) = config.iter().find(|&c| c.library_name == source) {
          let mut rm_specifier = HashSet::default();

          for (specifier_idx, specifier) in var.specifiers.iter().enumerate() {
            match specifier {
              ImportSpecifier::Named(ref s) => {
                let imported = s.imported.as_ref().map(|imported| match imported {
                  ModuleExportName::Ident(ident) => ident.sym.to_string(),
                  ModuleExportName::Str(str) => str.value.to_string(),
                });

                let as_name: Option<String> = imported.is_some().then(|| s.local.sym.to_string());
                let ident: String = imported.unwrap_or_else(|| s.local.sym.to_string());

                let mark = s.local.span.ctxt.as_u32();

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
                  continue;
                } else {
                  // not referenced, should tree shaking
                  rm_specifier.insert(specifier_idx);
                }
              }
              ImportSpecifier::Default(ref _s) => {}
              ImportSpecifier::Namespace(ref _ns) => {}
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
              span: Span::new(
                BytePos::DUMMY,
                BytePos::DUMMY,
                SyntaxContext::from_u32(js_source.mark),
              ),
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
                sym: Atom::from(js_source.default_spec.as_str()),
                optional: false,
              }))
            } else {
              None
            },
            local: Ident {
              span: Span::new(
                BytePos::DUMMY,
                BytePos::DUMMY,
                SyntaxContext::from_u32(js_source.mark),
              ),
              sym: Atom::from(js_source.as_name.unwrap_or(js_source.default_spec).as_str()),
              optional: false,
            },
            is_type_only: false,
          })]
        },
        src: Box::new(Str {
          span: DUMMY_SP,
          value: Atom::from(js_source_ref),
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
          value: Atom::from(css_source),
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

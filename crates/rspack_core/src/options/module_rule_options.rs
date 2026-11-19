use std::sync::Arc;

use rspack_util::{MergeFrom, fx_hash::FxDashMap};
use smallvec::SmallVec;

use crate::{
  AssetInlineGeneratorOptions, AssetResourceGeneratorOptions, CssAutoOrModuleParserOptions,
  CssModuleGeneratorOptions, CssModuleParserOptions, GeneratorOptions, ModuleOptions,
  ModuleRuleEffectIndices, ModuleType, ParserOptions, ParserOptionsMap, Resolve,
};

#[derive(Debug)]
pub struct ModuleRuleOptions {
  pub resolve: Option<Arc<Resolve>>,
  pub parser: Option<Arc<ParserOptions>>,
  pub generator: Option<Arc<GeneratorOptions>>,
}

// Usually the rules determine a single module type. Inline requests can override it.
type OptionsByType = SmallVec<[(ModuleType, Arc<ModuleRuleOptions>); 1]>;

#[derive(Debug, Default)]
pub struct ModuleRuleOptionsStore(FxDashMap<ModuleRuleEffectIndices, OptionsByType>);

impl ModuleOptions {
  /// Lazily resolve options, including when a module is restored from the filesystem cache.
  pub fn get_rule_options(
    &self,
    indices: &ModuleRuleEffectIndices,
    module_type: ModuleType,
  ) -> Arc<ModuleRuleOptions> {
    let find = |entries: &OptionsByType| {
      entries
        .iter()
        .find(|(ty, _)| *ty == module_type)
        .map(|(_, options)| Arc::clone(options))
    };
    if let Some(options) = self
      .options_store
      .0
      .get(indices)
      .and_then(|entry| find(&entry))
    {
      return options;
    }
    let mut entries = self.options_store.0.entry(indices.clone()).or_default();
    if let Some(options) = find(&entries) {
      return options;
    }

    let mut local_parser = None;
    let mut local_generator = None;
    let mut resolve: Option<Resolve> = None;
    for &idx in indices {
      let effect = &self.effects[idx];
      local_parser = local_parser.merge_from(&effect.parser);
      local_generator = local_generator.merge_from(&effect.generator);
      if entries.is_empty()
        && let Some(next) = &effect.resolve
      {
        resolve = Some(match resolve {
          Some(previous) => previous.merge(next.clone()),
          None => next.clone(),
        });
      }
    }
    let resolve = match entries.first() {
      Some((_, options)) => options.resolve.clone(),
      None => resolve.map(Arc::new),
    };
    let global_parser = self
      .parser
      .as_ref()
      .and_then(|map| resolve_global_parser_options(map, &module_type));
    let global_generator = self
      .generator
      .as_ref()
      .and_then(|map| resolve_global_generator_options(map, &module_type));
    let parser = merge_parser_options_with_local(global_parser, local_parser);
    let generator = merge_generator_options_with_local(global_generator, local_generator);
    let options = Arc::new(ModuleRuleOptions {
      resolve,
      parser: normalize_css_parser_options(&module_type, parser).map(Arc::new),
      generator: normalize_css_generator_options(&module_type, generator).map(Arc::new),
    });
    entries.push((module_type, Arc::clone(&options)));
    options
  }
}

fn resolve_global_generator_options(
  generator_options: &crate::GeneratorOptionsMap,
  module_type: &ModuleType,
) -> Option<GeneratorOptions> {
  let options = generator_options.get(module_type.as_str());
  match module_type {
    ModuleType::AssetInline | ModuleType::AssetResource => rspack_util::merge_from_optional_with(
      generator_options.get("asset").cloned(),
      options,
      |asset_options, options| match (asset_options, options) {
        (GeneratorOptions::Asset(a), GeneratorOptions::AssetInline(b)) => {
          GeneratorOptions::AssetInline(Into::<AssetInlineGeneratorOptions>::into(a).merge_from(b))
        }
        (GeneratorOptions::Asset(a), GeneratorOptions::AssetResource(b)) => {
          GeneratorOptions::AssetResource(
            Into::<AssetResourceGeneratorOptions>::into(a).merge_from(b),
          )
        }
        _ => unreachable!(),
      },
    ),
    ModuleType::CssAuto | ModuleType::CssGlobal | ModuleType::CssModule => {
      rspack_util::merge_from_optional_with(
        generator_options.get("css").cloned(),
        options,
        |css_options, options| match (css_options, options) {
          (GeneratorOptions::Css(a), GeneratorOptions::CssModule(b)) => {
            GeneratorOptions::CssModule(Into::<CssModuleGeneratorOptions>::into(&a).merge_from(b))
          }
          _ => unreachable!(),
        },
      )
    }
    _ => options.cloned(),
  }
}

fn resolve_global_parser_options(
  parser_options: &ParserOptionsMap,
  module_type: &ModuleType,
) -> Option<ParserOptions> {
  let options = parser_options.get(module_type.as_str());
  match module_type {
    ModuleType::JsAuto | ModuleType::JsDynamic | ModuleType::JsEsm => {
      // Merge `module.parser.["javascript/xxx"]` with `module.parser.["javascript"]` first.
      rspack_util::merge_from_optional_with(
        parser_options.get("javascript").cloned(),
        options,
        |javascript_options, options| match (javascript_options, options) {
          (
            ParserOptions::Javascript(a),
            ParserOptions::JavascriptAuto(b)
            | ParserOptions::JavascriptDynamic(b)
            | ParserOptions::JavascriptEsm(b),
          ) => ParserOptions::Javascript(a.merge_from(b)),
          _ => unreachable!(),
        },
      )
    }
    ModuleType::CssAuto | ModuleType::CssGlobal | ModuleType::CssModule => {
      rspack_util::merge_from_optional_with(
        parser_options.get("css").cloned(),
        options,
        |css_options, options| match (css_options, options) {
          (ParserOptions::Css(a), ParserOptions::CssAutoOrModule(b)) => {
            ParserOptions::CssAutoOrModule(
              Into::<CssAutoOrModuleParserOptions>::into(&a).merge_from(b),
            )
          }
          (ParserOptions::Css(a), ParserOptions::CssModule(b)) => ParserOptions::CssAutoOrModule(
            Into::<CssModuleParserOptions>::into(&a)
              .merge_from(b)
              .into(),
          ),
          _ => unreachable!(),
        },
      )
    }
    _ => options.cloned(),
  }
}

fn merge_parser_options_with_local(
  global_parser: Option<ParserOptions>,
  local_parser: Option<ParserOptions>,
) -> Option<ParserOptions> {
  match (global_parser, local_parser) {
    (None, options) | (options, None) => options,
    (Some(global), Some(local)) => Some(match (global, &local) {
      (ParserOptions::Asset(a), ParserOptions::Asset(b)) => ParserOptions::Asset(a.merge_from(b)),
      (ParserOptions::Css(a), ParserOptions::Css(b)) => ParserOptions::Css(a.merge_from(b)),
      (ParserOptions::CssModule(a), ParserOptions::CssModule(b)) => {
        ParserOptions::CssModule(a.merge_from(b))
      }
      (ParserOptions::CssAutoOrModule(a), ParserOptions::CssAutoOrModule(b)) => {
        ParserOptions::CssAutoOrModule(a.merge_from(b))
      }
      (
        ParserOptions::Javascript(a),
        ParserOptions::JavascriptAuto(b)
        | ParserOptions::JavascriptDynamic(b)
        | ParserOptions::JavascriptEsm(b),
      ) => ParserOptions::Javascript(a.merge_from(b)),
      (ParserOptions::Json(a), ParserOptions::Json(b)) => ParserOptions::Json(a.merge_from(b)),
      (global, _) => global,
    }),
  }
}

fn merge_generator_options_with_local(
  global_generator: Option<GeneratorOptions>,
  local_generator: Option<GeneratorOptions>,
) -> Option<GeneratorOptions> {
  match (global_generator, local_generator) {
    (None, options) | (options, None) => options,
    (Some(global), Some(local)) => Some(match (&global, &local) {
      (GeneratorOptions::Asset(_), GeneratorOptions::Asset(_))
      | (GeneratorOptions::AssetInline(_), GeneratorOptions::AssetInline(_))
      | (GeneratorOptions::AssetResource(_), GeneratorOptions::AssetResource(_))
      | (GeneratorOptions::Css(_), GeneratorOptions::Css(_))
      | (GeneratorOptions::CssModule(_), GeneratorOptions::CssModule(_))
      | (GeneratorOptions::Json(_), GeneratorOptions::Json(_)) => global.merge_from(&local),
      _ => global,
    }),
  }
}

fn normalize_css_parser_options(
  module_type: &ModuleType,
  parser: Option<ParserOptions>,
) -> Option<ParserOptions> {
  if !matches!(
    module_type,
    ModuleType::Css | ModuleType::CssAuto | ModuleType::CssGlobal | ModuleType::CssModule
  ) {
    return parser;
  }

  match parser.as_ref() {
    Some(ParserOptions::Css(options)) => Some(ParserOptions::CssAutoOrModule(
      CssAutoOrModuleParserOptions::from(options),
    )),
    Some(ParserOptions::CssModule(options)) => Some(ParserOptions::CssAutoOrModule(
      CssAutoOrModuleParserOptions::from(options.clone()),
    )),
    _ => parser,
  }
}

fn normalize_css_generator_options(
  module_type: &ModuleType,
  generator: Option<GeneratorOptions>,
) -> Option<GeneratorOptions> {
  if !matches!(
    module_type,
    ModuleType::Css | ModuleType::CssAuto | ModuleType::CssGlobal | ModuleType::CssModule
  ) {
    return generator;
  }

  match generator.as_ref() {
    Some(GeneratorOptions::Css(options)) => Some(GeneratorOptions::CssModule(
      CssModuleGeneratorOptions::from(options),
    )),
    _ => generator,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    AssetGeneratorOptions, CssGeneratorOptions, CssParserOptions, JavascriptParserCreateRequire,
    JavascriptParserOptions,
  };

  #[test]
  fn reuse_global_parser_options_when_local_options_are_empty() {
    let global = ParserOptions::Javascript(JavascriptParserOptions {
      jsx: Some(true),
      ..Default::default()
    });

    let resolved = merge_parser_options_with_local(Some(global), None)
      .expect("global parser options should be reused");

    assert_eq!(
      resolved
        .get_javascript()
        .expect("javascript parser options should be resolved")
        .jsx,
      Some(true)
    );
  }

  #[test]
  fn merge_local_parser_options_into_global_parser_options() {
    let global = ParserOptions::Javascript(JavascriptParserOptions {
      jsx: Some(true),
      require_dynamic: Some(true),
      ..Default::default()
    });
    let local = ParserOptions::JavascriptAuto(JavascriptParserOptions {
      jsx: Some(false),
      ..Default::default()
    });

    let resolved = merge_parser_options_with_local(Some(global), Some(local))
      .expect("merged parser options should exist");

    let options = resolved
      .get_javascript()
      .expect("javascript parser options should be resolved");
    assert_eq!(options.jsx, Some(false));
    assert_eq!(options.require_dynamic, Some(true));
  }

  #[test]
  fn reuse_global_generator_options_when_local_options_are_empty() {
    let global_generator = GeneratorOptions::Asset(AssetGeneratorOptions {
      emit: Some(true),
      ..Default::default()
    });

    let module = ModuleOptions {
      effects: vec![crate::ModuleRuleEffect {
        resolve: Some(Resolve::default()),
        ..Default::default()
      }],
      generator: Some([("asset".into(), global_generator)].into_iter().collect()),
      ..Default::default()
    };
    let indices = smallvec::smallvec![0];
    let options = module.get_rule_options(&indices, ModuleType::Asset);
    assert!(Arc::ptr_eq(
      &options,
      &module.get_rule_options(&indices, ModuleType::Asset)
    ));
    let js_options = module.get_rule_options(&indices, ModuleType::JsAuto);
    assert!(js_options.generator.is_none());
    assert!(Arc::ptr_eq(
      options.resolve.as_ref().unwrap(),
      js_options.resolve.as_ref().unwrap()
    ));
    assert_eq!(
      options
        .generator
        .as_deref()
        .and_then(GeneratorOptions::get_asset)
        .expect("asset generator options should exist")
        .emit,
      Some(true)
    );
  }

  #[test]
  fn normalize_css_options_for_parser_and_generator_builder() {
    for module_type in [
      ModuleType::Css,
      ModuleType::CssAuto,
      ModuleType::CssGlobal,
      ModuleType::CssModule,
    ] {
      let parser = normalize_css_parser_options(
        &module_type,
        Some(ParserOptions::Css(CssParserOptions {
          export_type: None,
          named_exports: Some(true),
          url: None,
          r#import: None,
          resolve_import: None,
        })),
      );
      let generator = normalize_css_generator_options(
        &module_type,
        Some(GeneratorOptions::Css(CssGeneratorOptions {
          exports_only: Some(false),
          runtime_public_path: Some(false),
          es_module: Some(true),
        })),
      );

      let parser_options = parser
        .as_ref()
        .and_then(ParserOptions::get_css_module)
        .expect("css parser options should be normalized to CssAutoOrModule");
      let generator_options = generator
        .as_ref()
        .and_then(GeneratorOptions::get_css_module)
        .expect("css generator options should be normalized to CssModule");

      assert_eq!(parser_options.named_exports, Some(true));
      assert_eq!(generator_options.exports_only, Some(false));
      assert_eq!(generator_options.runtime_public_path, Some(false));
      assert_eq!(generator_options.es_module, Some(true));
    }
  }

  #[test]
  fn merge_explicit_disabled_create_require_into_global_parser_options() {
    let global = ParserOptions::Javascript(JavascriptParserOptions {
      create_require: Some(JavascriptParserCreateRequire::Enabled(
        "createRequire from module".to_string(),
      )),
      ..Default::default()
    });
    let local = ParserOptions::JavascriptAuto(JavascriptParserOptions {
      create_require: Some(JavascriptParserCreateRequire::Disabled),
      ..Default::default()
    });

    let resolved = merge_parser_options_with_local(Some(global), Some(local))
      .expect("merged parser options should exist");

    let options = resolved
      .get_javascript()
      .expect("javascript parser options should be resolved");
    assert!(!options.is_create_require_enabled());
  }
}

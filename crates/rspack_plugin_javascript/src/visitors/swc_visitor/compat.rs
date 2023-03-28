use either::Either;
use rspack_core::PresetEnv;
use swc_core::common::{chain, comments::SingleThreadedComments, pass::Optional, Mark};
use swc_core::ecma::ast::EsVersion;
use swc_core::ecma::preset_env as swc_ecma_preset_env;
use swc_core::ecma::transforms::base::{feature::FeatureFlag, pass::noop, Assumptions};
use swc_core::ecma::transforms::compat;
use swc_core::ecma::visit::Fold;

fn compat_by_preset_env(
  preset_env_config: Option<PresetEnv>,
  top_level_mark: Mark,
  assumptions: Assumptions,
  comments: Option<&SingleThreadedComments>,
) -> impl Fold + '_ {
  if let Some(PresetEnv { mode, targets, core_js }) = preset_env_config && !targets.is_empty() {
    let core_js = if let Some(core_js) = &core_js && let Ok(core_js) = core_js.parse() {
      Some(core_js)
    } else {
      None
    };

    Either::Left(swc_ecma_preset_env::preset_env(
      top_level_mark,
      comments,
      swc_ecma_preset_env::Config {
        mode: mode.map(Into::into),
        targets: if targets.is_empty() {
          None
        } else {
          Some(swc_ecma_preset_env::Targets::Query(
            preset_env_base::query::Query::Multiple(targets),
          ))
        },
        core_js,
        ..Default::default()
      },
      assumptions,
      &mut FeatureFlag::empty(),
    ))
  } else {
    Either::Right(noop())
  }
}

fn compat_by_es_version(
  es_version: Option<EsVersion>,
  unresolved_mark: Mark,
  assumptions: Assumptions,
  comments: Option<&SingleThreadedComments>,
  is_typescript: bool,
) -> impl Fold + '_ {
  if let Some(es_version) = es_version {
    dbg!(&es_version);
    dbg!(&assumptions);
    Either::Left(chain!(
      // Optional::new(
      //   compat::class_fields_use_set::class_fields_use_set(assumptions.pure_getters),
      //   assumptions.set_public_class_fields,
      // ),
      // Optional::new(
      //   compat::es2022::es2022(
      //     comments,
      //     compat::es2022::Config {
      //       class_properties: compat::es2022::class_properties::Config {
      //         private_as_properties: assumptions.private_fields_as_properties,
      //         constant_super: assumptions.constant_super,
      //         set_public_fields: assumptions.set_public_class_fields,
      //         no_document_all: assumptions.no_document_all,
      //         static_blocks_mark: Mark::new()
      //       }
      //     }
      //   ),
      //   es_version < EsVersion::Es2022
      // ),
      // Optional::new(compat::es2021::es2021(), es_version < EsVersion::Es2021),
      // Optional::new(
      //   compat::es2020::es2020(compat::es2020::Config {
      //     nullish_coalescing: compat::es2020::nullish_coalescing::Config {
      //       no_document_all: assumptions.no_document_all
      //     },
      //     optional_chaining: compat::es2020::opt_chaining::Config {
      //       no_document_all: assumptions.no_document_all,
      //       pure_getter: assumptions.pure_getters
      //     }
      //   }),
      //   es_version < EsVersion::Es2020
      // ),
      // Optional::new(compat::es2019::es2019(), es_version < EsVersion::Es2019),
      // Optional::new(
      //   compat::es2018(compat::es2018::Config {
      //     object_rest_spread: compat::es2018::object_rest_spread::Config {
      //       no_symbol: assumptions.object_rest_no_symbols,
      //       set_property: assumptions.set_spread_properties,
      //       pure_getters: assumptions.pure_getters
      //     }
      //   }),
      //   es_version < EsVersion::Es2018
      // ),
      // Optional::new(
      //   compat::es2017(
      //     compat::es2017::Config {
      //       async_to_generator: compat::es2017::async_to_generator::Config {
      //         // ignore_function_name: assumptions.ignore_function_name,
      //         ignore_function_name: true,
      //         ignore_function_length: true
      //       },
      //     },
      //     comments,
      //     unresolved_mark
      //   ),
      //   es_version < EsVersion::Es2017
      // ),
      Optional::new(compat::es2016(), es_version < EsVersion::Es2016),
      Optional::new(
        compat::es2015(
          unresolved_mark,
          comments,
          compat::es2015::Config {
            classes: compat::es2015::classes::Config {
              constant_super: assumptions.constant_super,
              no_class_calls: assumptions.no_class_calls,
              set_class_methods: assumptions.set_class_methods,
              super_is_callable_constructor: assumptions.super_is_callable_constructor
            },
            computed_props: compat::es2015::computed_props::Config { loose: false },
            for_of: compat::es2015::for_of::Config {
              assume_array: false,
              loose: false
            },
            spread: compat::es2015::spread::Config { loose: false },
            destructuring: compat::es2015::destructuring::Config { loose: false },
            regenerator: Default::default(),
            template_literal: compat::es2015::template_literal::Config {
              ignore_to_primitive: assumptions.ignore_to_primitive_hint,
              mutable_template: assumptions.mutable_template_object
            },
            parameters: compat::es2015::parameters::Config {
              ignore_function_length: assumptions.ignore_function_length,
            },
            typescript: is_typescript
          }
        ),
        es_version < EsVersion::Es2015
      ),
      Optional::new(compat::es3(true), es_version == EsVersion::Es3),
      Optional::new(compat::es3(true), es_version == EsVersion::Es3)
    ))
  } else {
    Either::Right(noop())
  }
}

pub fn compat(
  preset_env_config: Option<PresetEnv>,
  es_version: Option<EsVersion>,
  assumptions: Assumptions,
  top_level_mark: Mark,
  unresolved_mark: Mark,
  comments: Option<&SingleThreadedComments>,
  is_typescript: bool,
) -> impl Fold + '_ {
  chain!(
    compat_by_preset_env(preset_env_config, top_level_mark, assumptions, comments),
    compat_by_es_version(
      es_version,
      unresolved_mark,
      assumptions,
      comments,
      is_typescript
    )
  )
}

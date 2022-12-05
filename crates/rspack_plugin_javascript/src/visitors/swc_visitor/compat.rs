use either::Either;
use swc_core::common::{chain, comments::SingleThreadedComments, pass::Optional, Mark};
use swc_core::ecma::ast::EsVersion;
use swc_core::ecma::transforms::base::{feature::FeatureFlag, pass::noop, Assumptions};
use swc_core::ecma::transforms::compat;
use swc_core::ecma::visit::Fold;

use swc_core::ecma::preset_env as swc_ecma_preset_env;

type BrowserConfig = (Vec<String>, bool);

fn compat_by_browser_list(
  browser_config: Option<BrowserConfig>,
  top_level_mark: Mark,
  assumptions: Assumptions,
  comments: Option<&SingleThreadedComments>,
) -> impl Fold + '_ {
  if let Some((browserlist, polyfill)) = browser_config {
    Either::Left(swc_ecma_preset_env::preset_env(
      top_level_mark,
      comments,
      swc_ecma_preset_env::Config {
        mode: if polyfill {
          Some(swc_ecma_preset_env::Mode::Usage)
        } else {
          Some(swc_ecma_preset_env::Mode::Entry)
        },
        targets: Some(swc_ecma_preset_env::Targets::Query(
          preset_env_base::query::Query::Multiple(browserlist),
        )),
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
    Either::Left(chain!(
      Optional::new(
        compat::es2022::es2022(
          comments,
          compat::es2022::Config {
            class_properties: compat::es2022::class_properties::Config {
              private_as_properties: assumptions.private_fields_as_properties,
              constant_super: assumptions.constant_super,
              set_public_fields: assumptions.set_public_class_fields,
              no_document_all: assumptions.no_document_all
            }
          }
        ),
        es_version < EsVersion::Es2022
      ),
      Optional::new(compat::es2021::es2021(), es_version < EsVersion::Es2021),
      Optional::new(
        compat::es2020::es2020(compat::es2020::Config {
          nullish_coalescing: compat::es2020::nullish_coalescing::Config {
            no_document_all: assumptions.no_document_all
          },
          optional_chaining: compat::es2020::opt_chaining::Config {
            no_document_all: assumptions.no_document_all,
            pure_getter: assumptions.pure_getters
          }
        }),
        es_version < EsVersion::Es2020
      ),
      Optional::new(compat::es2019::es2019(), es_version < EsVersion::Es2019),
      Optional::new(
        compat::es2018(compat::es2018::Config {
          object_rest_spread: compat::es2018::object_rest_spread::Config {
            no_symbol: assumptions.object_rest_no_symbols,
            set_property: assumptions.set_spread_properties,
            pure_getters: assumptions.pure_getters
          }
        }),
        es_version < EsVersion::Es2018
      ),
      Optional::new(
        compat::es2017(
          compat::es2017::Config {
            async_to_generator: compat::es2017::async_to_generator::Config {
              ignore_function_name: assumptions.ignore_function_name,
              ignore_function_length: assumptions.ignore_function_length
            },
          },
          comments,
          unresolved_mark
        ),
        es_version < EsVersion::Es2017
      ),
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
              ..Default::default()
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
      Optional::new(compat::es3(true), es_version == EsVersion::Es3)
    ))
  } else {
    Either::Right(noop())
  }
}

pub fn compat(
  browser_config: Option<BrowserConfig>,
  es_version: Option<EsVersion>,
  top_level_mark: Mark,
  unresolved_mark: Mark,
  comments: Option<&SingleThreadedComments>,
  is_typescript: bool,
) -> impl Fold + '_ {
  let mut assumptions = Assumptions::default();
  if is_typescript {
    assumptions.set_class_methods = true;
    assumptions.set_public_class_fields = true;
  };

  chain!(
    compat_by_browser_list(browser_config, top_level_mark, assumptions, comments),
    compat_by_es_version(
      es_version,
      unresolved_mark,
      assumptions,
      comments,
      is_typescript
    )
  )
}

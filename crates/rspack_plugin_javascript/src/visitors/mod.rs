mod dependency;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use dependency::*;
use either::Either;
use swc_core::common::comments::Comments;
use swc_core::ecma::visit::Fold;
use xxhash_rust::xxh32::xxh32;
mod clear_mark;

use rspack_core::{BuildInfo, ModuleType};
use swc_core::ecma::transforms::base::Assumptions;
pub mod swc_visitor;
use rspack_ast::javascript::Ast;
use rspack_core::{CompilerOptions, ResourceData};
use rspack_error::{AnyhowError, Result};
use swc_core::common::{chain, Mark, SourceMap};
use swc_core::ecma::parser::Syntax;
use swc_core::ecma::transforms::base::pass::{noop, Optional};

macro_rules! either {
  ($config:expr, $f:expr) => {
    if let Some(config) = &$config {
      #[allow(clippy::redundant_closure_call)]
      Either::Left($f(config))
    } else {
      Either::Right(noop())
    }
  };
}

/// Rspack builtins
/// - `react`: a port of `react-refresh/babel`, and `react-fast-refresh-loader`
/// - `relay`: a port of `babel-plugin-relay`
/// - `import`: a port of `babel-plugin-import`
/// - `emotion`: a port of `babel-plugin-emotion`
// #[deprecated(
//   note = "Builtin transform is deprecated and will be removed in JS version 0.5.0, see: https://github.com/web-infra-dev/rspack/pull/4133"
// )]
#[allow(clippy::too_many_arguments)]
fn builtins_additional_feature_transforms<'b>(
  resource_data: &'b ResourceData,
  options: &'b CompilerOptions,
  module_type: &'b ModuleType,
  source: &'b str,
  top_level_mark: Mark,
  unresolved_mark: Mark,
  cm: Arc<SourceMap>,
) -> impl Fold + 'b {
  let comments = None;
  // TODO: should use react-loader to get exclude/include
  let should_transform_by_react = module_type.is_jsx_like();

  chain!(
    Optional::new(
      rspack_swc_visitors::react(
        top_level_mark,
        comments,
        &cm,
        &options.builtins.react,
        unresolved_mark
      ),
      should_transform_by_react
    ),
    Optional::new(
      rspack_swc_visitors::fold_react_refresh(unresolved_mark),
      should_transform_by_react && options.builtins.react.refresh.unwrap_or_default()
    ),
    either!(
      options.builtins.emotion,
      |emotion_options: &rspack_swc_visitors::EmotionOptions| {
        rspack_swc_visitors::emotion(
          emotion_options.clone(),
          &resource_data.resource_path,
          xxh32(source.as_bytes(), 0),
          cm.clone(),
          comments,
        )
      }
    ),
    either!(options.builtins.relay, |relay_option| {
      rspack_swc_visitors::relay(
        relay_option,
        &resource_data.resource_path,
        PathBuf::from(AsRef::<Path>::as_ref(&options.context)),
        unresolved_mark,
      )
    }),
    either!(options.builtins.plugin_import, |config| {
      swc_plugin_import::plugin_import(config)
    })
  )
}

/// Webpack builtin plugins
/// - `define`: a port of `DefinePlugin`
/// - `provide`: a port of `ProvidePlugin`
fn builtins_webpack_plugin(options: &CompilerOptions, unresolved_mark: Mark) -> impl Fold + '_ {
  chain!(
    Optional::new(
      rspack_swc_visitors::define(&options.builtins.define),
      !options.builtins.define.is_empty()
    ),
    Optional::new(
      builtins_webpack_plugin_define_optimizer(unresolved_mark),
      !options.builtins.define.is_empty()
    ),
    Optional::new(
      rspack_swc_visitors::provide(&options.builtins.provide, unresolved_mark),
      !options.builtins.provide.is_empty()
    )
  )
}

/// Removing this will **not** cause semantic difference, but will avoid overheads that are not necessary.
/// Expression simplifier and dead branch remover are working together to avoid parsing not necessary path.
///
/// Example:
/// ```js
/// if(process.env.NODE_ENV === "development") {
///   module.exports = require(".../xxx.development.js")
/// } else {
///   module.exports = require(".../xxx.production.js")
/// }
/// ```
/// The ordering of these two is important, `expr_simplifier` goes first and `dead_branch_remover` goes second.
///
/// `if(process.env.NODE_ENV === "development")` -> <define> -> `if("development" === "development")`
/// `if("development" === "development") {} else {}` -> <expr_simplifier> -> `if(true) {} else {}`
/// `if(true) {} else {}` -> <dead_branch_remover> -> `else` branch is removed
fn builtins_webpack_plugin_define_optimizer(unresolved_mark: Mark) -> impl Fold {
  chain!(
    swc_visitor::expr_simplifier(unresolved_mark, Default::default()),
    swc_visitor::dead_branch_remover(unresolved_mark)
  )
}

#[allow(clippy::too_many_arguments)]
// #[deprecated(
//   note = "Compat transform is deprecated and will be removed in JS version 0.5.0, see: https://github.com/web-infra-dev/rspack/discussions/4070"
// )]
fn compat_transform<'b>(
  resource_data: &'b ResourceData,
  options: &'b CompilerOptions,
  top_level_mark: Mark,
  unresolved_mark: Mark,
  assumptions: Assumptions,
  syntax: Syntax,
) -> impl Fold + 'b {
  let es_version = match options.target.es_version {
    rspack_core::TargetEsVersion::Esx(es_version) => Some(es_version),
    _ => None,
  };
  let resource_path = resource_data.resource_path.to_string_lossy();

  chain!(
    Optional::new(
      swc_visitor::export_default_from(),
      syntax.export_default_from()
    ),
    swc_visitor::paren_remover(None),
    Optional::new(
      swc_visitor::compat(
        options.builtins.preset_env.clone(),
        es_version,
        assumptions,
        top_level_mark,
        unresolved_mark,
        None,
        syntax.typescript(),
      ),
      !resource_path.contains("@swc/helpers")
        && !resource_path.contains("tslib")
        && !resource_path.contains("core-js"),
    ),
    swc_visitor::reserved_words(),
  )
}

#[allow(clippy::too_many_arguments)]
pub fn run_before_pass(
  resource_data: &ResourceData,
  ast: &mut Ast,
  options: &CompilerOptions,
  syntax: Syntax,
  _build_info: &mut BuildInfo,
  module_type: &ModuleType,
  source: &str,
) -> Result<()> {
  println!("{}", source);
  let cm = ast.get_context().source_map.clone();
  ast
    .transform_with_handler(cm.clone(), |_handler, program, context| {
      let top_level_mark = context.top_level_mark;
      let unresolved_mark = context.unresolved_mark;
      let comments: Option<&dyn Comments> = None;

      let mut assumptions = Assumptions::default();
      if syntax.typescript() {
        assumptions.set_class_methods = true;
        assumptions.set_public_class_fields = true;
      }

      let mut pass = chain!(
        swc_visitor::resolver(unresolved_mark, top_level_mark, syntax.typescript()),
        Optional::new(
          swc_visitor::decorator(assumptions, &options.builtins.decorator),
          // Decorator transformation varies from `ModuleType`,
          // - TypeScript-like: decorators will be transformed by default, with legacy settings and will emit meta data.
          //      Since this is a default behavior with tsc.
          // - JavaScript-like: decorators will not be transformed(if `disableTransformByDefault` is on), and the parse will fail.
          (options.should_transform_by_default() || syntax.typescript()) && syntax.decorators()
        ),
        Optional::new(
          swc_visitor::typescript(top_level_mark, comments, &cm),
          options.should_transform_by_default() && syntax.typescript()
        ),
        Optional::new(
          builtins_additional_feature_transforms(
            resource_data,
            options,
            module_type,
            source,
            top_level_mark,
            unresolved_mark,
            cm
          ),
          options.should_transform_by_default()
        ),
        Optional::new(
          compat_transform(
            resource_data,
            options,
            top_level_mark,
            unresolved_mark,
            assumptions,
            syntax
          ),
          options.should_transform_by_default()
        ),
        builtins_webpack_plugin(options, unresolved_mark),
        // This will be deprecated in the future when TypeScript support and transform by default is dropped.
        // These features are coupled with this.
        swc_visitor::inject_helpers(unresolved_mark),
        swc_visitor::hygiene(false, top_level_mark),
        swc_visitor::fixer(comments.map(|v| v as &dyn Comments)),
      );
      program.fold_with(&mut pass);

      Ok(())
    })
    .map_err(AnyhowError::from)?;

  Ok(())
}

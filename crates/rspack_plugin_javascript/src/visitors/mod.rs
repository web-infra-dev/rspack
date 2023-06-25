mod dependency;
use std::path::{Path, PathBuf};

pub use dependency::*;
use either::Either;
use xxhash_rust::xxh32::xxh32;
mod clear_mark;

mod plugin_import;

use rspack_core::{BuildInfo, ModuleType};
use swc_core::ecma::transforms::base::Assumptions;
pub mod relay;
pub mod swc_visitor;
use rspack_core::{ast::javascript::Ast, CompilerOptions, ResourceData};
use rspack_error::Result;
use swc_core::common::{chain, comments::Comments};
use swc_core::ecma::parser::Syntax;
use swc_core::ecma::transforms::base::pass::{noop, Optional};
use swc_emotion::EmotionOptions;

use crate::visitors::plugin_import::plugin_import;
use crate::visitors::relay::relay;

macro_rules! either {
  ($config: expr, $f: expr) => {
    if let Some(config) = &$config {
      Either::Left($f(config))
    } else {
      Either::Right(noop())
    }
  };
}

/// return (ast, top_level_mark, unresolved_mark, globals)
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
  let es_version = match options.target.es_version {
    rspack_core::TargetEsVersion::Esx(es_version) => Some(es_version),
    _ => None,
  };
  let cm = ast.get_context().source_map.clone();
  // TODO: should use react-loader to get exclude/include
  let should_transform_by_react = module_type.is_jsx_like();
  ast.transform_with_handler(cm.clone(), |_handler, program, context| {
    let top_level_mark = context.top_level_mark;
    let unresolved_mark = context.unresolved_mark;
    let comments = None;

    let mut assumptions = Assumptions::default();
    if syntax.typescript() {
      assumptions.set_class_methods = true;
      assumptions.set_public_class_fields = true;
    }

    let resource_path = resource_data.resource_path.to_string_lossy();

    let mut pass = chain!(
      swc_visitor::resolver(unresolved_mark, top_level_mark, syntax.typescript()),
      //      swc_visitor::lint(
      //        &ast,
      //        top_level_mark,
      //        unresolved_mark,
      //        EsVersion::Es2022,
      //        &cm
      //      ),
      Optional::new(
        swc_visitor::decorator(assumptions, &options.builtins.decorator),
        syntax.decorators()
      ),
      //    swc_visitor::import_assertions(),
      Optional::new(
        swc_visitor::typescript(assumptions, top_level_mark, comments, &cm),
        syntax.typescript()
      ),
      Optional::new(
        swc_visitor::react(
          top_level_mark,
          comments,
          &cm,
          &options.builtins.react,
          unresolved_mark
        ),
        should_transform_by_react
      ),
      Optional::new(
        swc_visitor::fold_react_refresh(),
        should_transform_by_react && options.builtins.react.refresh.is_some()
      ),
      either!(
        options.builtins.emotion,
        |emotion_options: &EmotionOptions| {
          swc_emotion::emotion(
            emotion_options.clone(),
            &resource_data.resource_path,
            xxh32(source.as_bytes(), 0),
            cm.clone(),
            comments,
          )
        }
      ),
      either!(options.builtins.relay, |relay_option| {
        relay(
          relay_option,
          resource_data.resource_path.as_path(),
          PathBuf::from(AsRef::<Path>::as_ref(&options.context)),
          unresolved_mark,
        )
      }),
      plugin_import(options.builtins.plugin_import.as_ref()),
      // enable if configurable
      // swc_visitor::const_modules(cm, globals),
      Optional::new(
        swc_visitor::define(&options.builtins.define),
        !options.builtins.define.is_empty()
      ),
      Optional::new(
        swc_visitor::provide_builtin(&options.builtins.provide, unresolved_mark),
        !options.builtins.provide.is_empty()
      ),
      Optional::new(
        swc_visitor::export_default_from(),
        syntax.export_default_from()
      ),
      // enable if necessary
      // swc_visitor::simplifier(unresolved_mark, Default::default()),
      // enable if configurable
      // swc_visitor::json_parse(min_cost),
      swc_visitor::paren_remover(comments.map(|v| v as &dyn Comments)),
      // es_version
      Optional::new(
        swc_visitor::compat(
          options.builtins.preset_env.clone(),
          es_version,
          assumptions,
          top_level_mark,
          unresolved_mark,
          comments,
          syntax.typescript()
        ),
        !resource_path.contains("@swc/helpers")
          && !resource_path.contains("tslib")
          && !resource_path.contains("core-js")
      ),
      swc_visitor::reserved_words(),
      swc_visitor::inject_helpers(unresolved_mark),
      // The ordering of these two is important, `expr_simplifier` goes first and `dead_branch_remover` goes second.
      swc_visitor::expr_simplifier(unresolved_mark, Default::default()),
      swc_visitor::dead_branch_remover(unresolved_mark),
      swc_visitor::fixer(comments.map(|v| v as &dyn Comments)),
    );
    program.fold_with(&mut pass);

    Ok(())
  })?;

  Ok(())
}

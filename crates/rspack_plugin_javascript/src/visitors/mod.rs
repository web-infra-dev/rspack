mod dependency_scanner;
pub use dependency_scanner::*;
mod finalize;
use finalize::finalize;
// mod clear_mark;
// use clear_mark::clear_mark;
mod inject_runtime_helper;
use inject_runtime_helper::inject_runtime_helper;
mod format;
use format::*;
mod pass_global;
use pass_global::PassGlobal;
use swc_common::{Globals, Mark};
mod swc_visitor;
use crate::utils::get_swc_compiler;
use anyhow::Error;
use rspack_core::{Compilation, CompilerOptions, ModuleGraphModule};
use swc::config::ModuleConfig;
use swc_common::{chain, comments::Comments};
use swc_ecma_ast::Program;
use swc_ecma_parser::Syntax;
use swc_ecma_transforms::modules::common_js::Config as CommonjsConfig;
use swc_ecma_transforms::pass::Optional;
use swc_ecma_visit::FoldWith;

/// return (ast, top_level_mark, unresolved_mark, globals)
pub fn run_before_pass(
  ast: Program,
  options: &CompilerOptions,
  syntax: Syntax,
) -> Result<(Program, Mark, Mark, Globals), Error> {
  let pass_global = PassGlobal::new();
  let top_level_mark = pass_global.top_level_mark;
  let unresolved_mark = pass_global.unresolved_mark;
  let cm = get_swc_compiler().cm.clone();
  let comments = None;
  let ret = pass_global.try_with_handler(move |handler| {
    let mut pass = chain!(
      swc_visitor::resolver(unresolved_mark, top_level_mark, syntax.typescript()),
      //      swc_visitor::lint(
      //        &ast,
      //        top_level_mark,
      //        unresolved_mark,
      //        EsVersion::Es2022,
      //        &cm
      //      ),
      Optional::new(swc_visitor::decorator(), syntax.decorators()),
      //    swc_visitor::import_assertions(),
      Optional::new(
        swc_visitor::typescript(top_level_mark, comments, &cm),
        syntax.typescript()
      ),
      Optional::new(
        swc_visitor::react(top_level_mark, comments, &cm, &options.builtins.react),
        syntax.jsx()
      ),
      // enable if configurable
      // swc_visitor::const_modules(cm, globals),
      Optional::new(
        swc_visitor::define(&options.builtins.define, handler, &cm),
        !options.builtins.define.is_empty()
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
      Optional::new(swc_visitor::private_in_object(), syntax.private_in_object()),
      swc_visitor::compat(
        if options.target.platform.is_browsers_list() {
          Some((
            options.builtins.browserslist.clone(),
            options.builtins.polyfill,
          ))
        } else {
          None
        },
        options.target.es_version,
        top_level_mark,
        unresolved_mark,
        comments,
        syntax.typescript()
      ),
      swc_visitor::reserved_words(),
      swc_visitor::inject_helpers(),
      swc_visitor::dead_branch_remover(unresolved_mark),
    );
    let ast = ast.fold_with(&mut pass);
    Ok(ast)
  });
  let globals = pass_global.global;
  ret.map(|ast| (ast, top_level_mark, unresolved_mark, globals))
}

pub fn run_after_pass(
  ast: Program,
  mgm: &ModuleGraphModule,
  compilation: &Compilation,
) -> Result<Program, Error> {
  let pass_global = PassGlobal::new();
  let unresolved_mark = pass_global.unresolved_mark;
  let cm = get_swc_compiler().cm.clone();
  let comments = None;
  pass_global.try_with_handler(|_handler| {
    let mut pass = chain!(
      swc_visitor::build_module(
        &cm,
        unresolved_mark,
        Some(ModuleConfig::CommonJs(CommonjsConfig {
          ignore_dynamic: true,
          strict_mode: false,
          no_interop: false,
          ..Default::default()
        })),
        comments,
        compilation.options.target.es_version
      ),
      inject_runtime_helper(),
      swc_visitor::hygiene(false),
      swc_visitor::fixer(comments.map(|v| v as &dyn Comments)),
      finalize(mgm, compilation, unresolved_mark)
    );

    Ok(ast.fold_with(&mut pass))
  })
}

mod dependency_scanner;
pub use dependency_scanner::*;
mod finalize;
pub(crate) mod minify;
use finalize::finalize;
// mod clear_mark;
// use clear_mark::clear_mark;
mod inject_runtime_helper;
use inject_runtime_helper::inject_runtime_helper;
mod format;
use format::*;
use swc_common::pass::Repeat;
mod swc_visitor;
mod tree_shaking;
use crate::utils::get_swc_compiler;
use rspack_core::{
  ast::javascript::Ast, Compilation, CompilerOptions, ModuleGraphModule, ResourceData,
};
use rspack_error::Result;
use swc::config::ModuleConfig;
use swc_common::{chain, comments::Comments};
use swc_ecma_parser::Syntax;
use swc_ecma_transforms::modules::common_js::Config as CommonjsConfig;
use swc_ecma_transforms::pass::Optional;
use swc_ecma_visit::FoldWith;
use tree_shaking::tree_shaker;
use ustr::ustr;

/// return (ast, top_level_mark, unresolved_mark, globals)
pub fn run_before_pass(
  resource_data: &ResourceData,
  ast: &mut Ast,
  options: &CompilerOptions,
  syntax: Syntax,
) -> Result<()> {
  let cm = get_swc_compiler().cm.clone();
  ast.transform_with_handler(cm.clone(), |handler, program, context| {
    let top_level_mark = context.top_level_mark;
    let unresolved_mark = context.unresolved_mark;
    let comments = None;
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
      Optional::new(
        {
          let context = &options.context;
          let uri = resource_data.resource.as_str();
          swc_visitor::fold_react_refresh(context, uri)
        },
        !resource_data.resource.contains("node_modules")
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
      // The ordering of these two is important, `expr_simplifier` goes first and `dead_branch_remover` goes second.
      Repeat::new(swc_visitor::expr_simplifier(
        unresolved_mark,
        Default::default()
      ),),
      swc_visitor::dead_branch_remover(unresolved_mark),
    );
    program.fold_with(&mut pass);
    Ok(())
  })?;
  Ok(())
}

pub fn run_after_pass(ast: &mut Ast, mgm: &ModuleGraphModule, compilation: &Compilation) {
  let cm = get_swc_compiler().cm.clone();
  ast.transform(|program, context| {
    let unresolved_mark = context.unresolved_mark;
    let comments = None;

    let mut pass = chain!(
      Optional::new(
        tree_shaker(
          ustr(&mgm.module_identifier,),
          &compilation.used_symbol,
          pass_global.top_level_mark,
          parse_phase_globals
        ),
        tree_shaking
      ),
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

    program.fold_with(&mut pass);
  });
}

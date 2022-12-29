mod dependency_scanner;
pub use dependency_scanner::*;
mod finalize;
use finalize::finalize;
// mod clear_mark;
// use clear_mark::clear_mark;
mod inject_runtime_helper;
use inject_runtime_helper::inject_runtime_helper;
mod strict;
use strict::strict_mode;
mod format;
use format::*;
mod module_variables;
use module_variables::*;
use rspack_core::{BuildInfo, Module, ModuleType};
use swc_core::common::pass::Repeat;
use swc_core::ecma::transforms::base::Assumptions;
use swc_core::ecma::transforms::optimization::simplify::dce::{dce, Config};
mod swc_visitor;
mod tree_shaking;
use rspack_core::{ast::javascript::Ast, CompilerOptions, GenerateContext, ResourceData};
use rspack_error::Result;
use swc_core::base::config::ModuleConfig;
use swc_core::common::{chain, comments::Comments};
use swc_core::ecma::parser::Syntax;
use swc_core::ecma::transforms::base::pass::Optional;
use swc_core::ecma::transforms::module::common_js::Config as CommonjsConfig;
use tree_shaking::tree_shaking_visitor;
use ustr::ustr;
/// return (ast, top_level_mark, unresolved_mark, globals)
pub fn run_before_pass(
  resource_data: &ResourceData,
  ast: &mut Ast,
  options: &CompilerOptions,
  syntax: Syntax,
  build_info: &mut BuildInfo,
  module_type: &ModuleType,
) -> Result<()> {
  let cm = ast.get_context().source_map.clone();
  // TODO: should use react-loader to get exclude/include
  let out_of_node_modules = !resource_data.resource.contains("node_modules");
  let should_transform_by_react = out_of_node_modules && module_type.is_jsx_like();
  ast.transform_with_handler(cm.clone(), |handler, program, context| {
    let top_level_mark = context.top_level_mark;
    let unresolved_mark = context.unresolved_mark;
    let comments = None;

    let mut assumptions = Assumptions::default();
    if syntax.typescript() {
      assumptions.set_class_methods = true;
      assumptions.set_public_class_fields = true;
    }

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
        swc_visitor::react(top_level_mark, comments, &cm, &options.builtins.react),
        should_transform_by_react
      ),
      Optional::new(
        {
          let context = &options.context;
          let uri = resource_data.resource.as_str();
          swc_visitor::fold_react_refresh(context, uri)
        },
        should_transform_by_react
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
        assumptions,
        top_level_mark,
        unresolved_mark,
        comments,
        syntax.typescript()
      ),
      swc_visitor::reserved_words(),
      swc_visitor::inject_helpers(),
      // The ordering of these two is important, `expr_simplifier` goes first and `dead_branch_remover` goes second.
      swc_visitor::expr_simplifier(unresolved_mark, Default::default()),
      swc_visitor::dead_branch_remover(unresolved_mark),
      strict_mode(build_info, context),
    );
    program.fold_with(&mut pass);

    Ok(())
  })?;

  Ok(())
}

pub fn run_after_pass(ast: &mut Ast, module: &dyn Module, generate_context: &mut GenerateContext) {
  let cm = ast.get_context().source_map.clone();

  ast.transform(|program, context| {
    let unresolved_mark = context.unresolved_mark;
    let top_level_mark = context.top_level_mark;
    let tree_shaking = generate_context.compilation.options.builtins.tree_shaking;
    let minify = generate_context.compilation.options.builtins.minify;
    let comments = None;

    let mut pass = chain!(
      Optional::new(
        tree_shaking_visitor(
          &generate_context.compilation.module_graph,
          ustr(&module.identifier()),
          &generate_context.compilation.used_symbol,
          &generate_context.compilation.used_indirect_symbol,
          top_level_mark,
        ),
        tree_shaking
      ),
      Optional::new(
        Repeat::new(dce(Config::default(), unresolved_mark)),
        // extra branch to avoid doing dce twice, (minify will exec dce)
        tree_shaking && !minify.enable,
      ),
      swc_visitor::build_module(
        &cm,
        unresolved_mark,
        Some(ModuleConfig::CommonJs(CommonjsConfig {
          ignore_dynamic: true,
          // here will remove `use strict`
          strict_mode: false,
          no_interop: !context.is_esm,
          ..Default::default()
        })),
        comments,
        generate_context.compilation.options.target.es_version
      ),
      inject_runtime_helper(unresolved_mark, generate_context.runtime_requirements),
      module_variables(
        module,
        unresolved_mark,
        top_level_mark,
        generate_context.compilation,
      ),
      finalize(module, generate_context.compilation, unresolved_mark),
      swc_visitor::hygiene(false),
      swc_visitor::fixer(comments.map(|v| v as &dyn Comments)),
    );

    program.fold_with(&mut pass);
  });
}

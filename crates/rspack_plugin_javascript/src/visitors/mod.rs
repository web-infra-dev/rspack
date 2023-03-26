mod dependency;
use std::collections::LinkedList;

pub use dependency::*;
mod finalize;
use either::Either;
use finalize::finalize;
mod clear_mark;

mod inject_runtime_helper;
use inject_runtime_helper::inject_runtime_helper;
mod plugin_import;
mod strict;

use strict::strict_mode;
mod format;
use format::*;
use rspack_core::{runtime_globals, BuildInfo, EsVersion, Module, ModuleType};
use swc_core::common::pass::Repeat;
use swc_core::ecma::transforms::base::Assumptions;
use swc_core::ecma::transforms::module::util::ImportInterop;
use swc_core::ecma::transforms::optimization::simplify::dce::{dce, Config};
pub mod relay;
mod swc_visitor;
mod tree_shaking;
use rspack_core::{
  ast::javascript::Ast, BuildMeta, CompilerOptions, GenerateContext, ResourceData,
};
use rspack_error::{Error, Result};
use swc_core::base::config::ModuleConfig;
use swc_core::common::{chain, comments::Comments};
use swc_core::ecma::parser::Syntax;
use swc_core::ecma::transforms::base::pass::{noop, Optional};
use swc_core::ecma::transforms::module::common_js::Config as CommonjsConfig;
use swc_emotion::EmotionOptions;
use tree_shaking::tree_shaking_visitor;
mod async_module;

use crate::visitors::async_module::build_async_module;
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
pub fn run_before_pass(
  resource_data: &ResourceData,
  ast: &mut Ast,
  options: &CompilerOptions,
  syntax: Syntax,
  build_info: &mut BuildInfo,
  build_meta: &mut BuildMeta,
  module_type: &ModuleType,
) -> Result<()> {
  let cm = ast.get_context().source_map.clone();
  // TODO: should use react-loader to get exclude/include
  let should_transform_by_react = module_type.is_jsx_like();
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
        should_transform_by_react && options.builtins.react.refresh.is_some()
      ),
      either!(
        options.builtins.emotion,
        |emotion_options: &EmotionOptions| {
          swc_emotion::emotion(
            emotion_options.clone(),
            &resource_data.resource_path,
            cm.clone(),
            comments,
          )
        }
      ),
      either!(options.builtins.relay, |relay_option| {
        relay(
          relay_option,
          resource_data.resource_path.as_path(),
          options.context.to_path_buf(),
          unresolved_mark,
        )
      }),
      plugin_import(options.builtins.plugin_import.as_ref()),
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
        options.builtins.preset_env.clone(),
        None,
        assumptions,
        top_level_mark,
        unresolved_mark,
        comments,
        syntax.typescript()
      ),
      swc_visitor::reserved_words(),
      swc_visitor::inject_helpers(unresolved_mark),
      // The ordering of these two is important, `expr_simplifier` goes first and `dead_branch_remover` goes second.
      swc_visitor::expr_simplifier(unresolved_mark, Default::default()),
      swc_visitor::dead_branch_remover(unresolved_mark),
      strict_mode(build_info, build_meta),
    );
    program.fold_with(&mut pass);

    Ok(())
  })?;

  Ok(())
}

pub fn run_after_pass(
  ast: &mut Ast,
  module: &dyn Module,
  generate_context: &mut GenerateContext,
) -> Result<()> {
  let cm = ast.get_context().source_map.clone();

  ast
    .transform_with_handler(cm.clone(), |_, program, context| {
      let unresolved_mark = context.unresolved_mark;
      let top_level_mark = context.top_level_mark;
      let compilation = generate_context.compilation;
      let builtin_tree_shaking = compilation.options.builtins.tree_shaking;
      let minify_options = &compilation.options.builtins.minify_options;
      let comments = None;
      let dependency_visitors =
        collect_dependency_code_generation_visitors(module, generate_context)?;
      let mgm = compilation
        .module_graph
        .module_graph_module_by_identifier(&module.identifier())
        .expect("should have module graph module");
      let need_tree_shaking = mgm.used;
      let build_meta = mgm.build_meta.as_ref().expect("should have build meta");
      let DependencyCodeGenerationVisitors {
        visitors,
        root_visitors,
        decl_mappings,
      } = dependency_visitors;

      {
        if !visitors.is_empty() {
          program.visit_mut_with_path(
            &mut DependencyVisitor::new(
              visitors
                .iter()
                .map(|(ast_path, visitor)| (ast_path, &**visitor))
                .collect(),
            ),
            &mut Default::default(),
          );
        }

        for (_, root_visitor) in root_visitors {
          program.visit_mut_with(&mut root_visitor.create());
        }
      }

      let mut promises = LinkedList::new();
      if build_meta.is_async {
        let runtime_requirements = &mut generate_context.runtime_requirements;
        runtime_requirements.insert(runtime_globals::MODULE);
        runtime_requirements.insert(runtime_globals::ASYNC_MODULE);
        decl_mappings.iter().for_each(|(_, referenced)| {
          promises.push_back(compilation.module_graph.is_async(referenced))
        });
      }

      let mut pass = chain!(
        Optional::new(
          tree_shaking_visitor(
            &decl_mappings,
            &compilation.module_graph,
            module.identifier(),
            &compilation.used_symbol_ref,
            top_level_mark,
            &compilation.side_effects_free_modules,
            &compilation.module_item_map,
            context.helpers.mark()
          ),
          builtin_tree_shaking && need_tree_shaking
        ),
        Optional::new(
          Repeat::new(dce(Config::default(), unresolved_mark)),
          need_tree_shaking && builtin_tree_shaking && minify_options.is_none()
        ),
        Optional::new(
          dce(Config::default(), unresolved_mark),
          need_tree_shaking && builtin_tree_shaking && minify_options.is_some()
        ),
        swc_visitor::build_module(
          &cm,
          unresolved_mark,
          Some(ModuleConfig::CommonJs(CommonjsConfig {
            ignore_dynamic: true,
            // here will remove `use strict`
            strict_mode: false,
            import_interop: if build_meta.strict_harmony_module {
              Some(ImportInterop::Node)
            } else if build_meta.esm {
              Some(ImportInterop::Swc)
            } else {
              None
            },
            allow_top_level_this: true,
            ..Default::default()
          })),
          comments,
          Some(EsVersion::Es5)
        ),
        Optional::new(build_async_module(promises), build_meta.is_async),
        inject_runtime_helper(unresolved_mark, generate_context.runtime_requirements),
        finalize(module, compilation, unresolved_mark),
        swc_visitor::hygiene(false, top_level_mark),
        swc_visitor::fixer(comments.map(|v| v as &dyn Comments)),
      );

      program.fold_with(&mut pass);

      Ok(())
    })
    .map_err(Error::from)
}

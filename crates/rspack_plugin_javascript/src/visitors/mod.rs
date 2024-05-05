mod dependency;
pub mod scope_info;
pub mod swc_visitor;

use rspack_ast::javascript::Ast;
use rspack_core::CompilerOptions;
use rspack_error::{AnyhowError, Result};
use swc_core::common::comments::Comments;
use swc_core::common::{chain, Mark};
use swc_core::ecma::transforms::base::pass::Optional;
use swc_core::ecma::visit::Fold;

pub use self::dependency::*;
use self::swc_visitor::dropped_comments_preserver;
pub use self::JavascriptParser;

/// Webpack builtin plugins
/// - `define`: a port of `DefinePlugin`
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
pub fn run_before_pass(ast: &mut Ast, options: &CompilerOptions) -> Result<()> {
  let cm = ast.get_context().source_map.clone();
  ast
    .transform_with_handler(cm.clone(), |_handler, program, context| {
      let top_level_mark = context.top_level_mark;
      let unresolved_mark = context.unresolved_mark;
      let comments = program.comments.take();
      {
        let mut pass = chain!(
          swc_visitor::resolver(unresolved_mark, top_level_mark, false),
          builtins_webpack_plugin(options, unresolved_mark),
          swc_visitor::hygiene(false, top_level_mark),
          swc_visitor::fixer(Some(&comments as &dyn Comments)),
          dropped_comments_preserver(comments.clone()),
        );
        program.fold_with(&mut pass);
      }
      program.comments = comments;
      Ok(())
    })
    .map_err(AnyhowError::from)?;

  Ok(())
}

use swc_core::ecma::ast::{Expr, Id, MemberExpr, OptChainExpr};

use super::{AllowedMemberTypes, JavascriptParser, MemberExpressionInfo};
use crate::visitors::{ExprRef, var_info::IdOrName};

/// callHooksForName/callHooksForInfo in webpack
/// webpack use HookMap and filter at callHooksForName/callHooksForInfo
/// we need to pass the name to hook to filter in the hook
pub trait CallHooks {
  fn call_hooks<F, T>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<T>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<T>;
}

impl CallHooks for Id {
  fn call_hooks<F, T>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<T>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<T>,
  {
    let id = self;
    let var_info = parser.get_var(id);
    let tags = var_info.tags().to_vec();
    for tag in tags {
      let tag_info = parser.definitions_db2.expect_get_tag_info(tag);
      parser.current_tag_info = Some(tag);
      let result = hook_call(parser, tag_info.tag);
      parser.current_tag_info = None;
      if result.is_some() {
        return result;
      }
    }

    if let Some(origin) = parser.get_var_origin(id).cloned() {
      let result = hook_call(parser, origin.name().as_str());
      if result.is_some() {
        return result;
      }
    }

    None
  }
}

impl CallHooks for IdOrName {
  fn call_hooks<F, T>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<T>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<T>,
  {
    match self {
      IdOrName::Id(id) => id.call_hooks(parser, hook_call),
      IdOrName::Name(name) => {
        // name of member expression, for example the `"import.meta"`.
        hook_call(parser, name.as_str())
      }
    }
  }
}

impl CallHooks for MemberExpr {
  fn call_hooks<F, T>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<T>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<T>,
  {
    let Some(MemberExpressionInfo::Expression(expr_name)) =
      parser.get_member_expression_info(ExprRef::Member(self), AllowedMemberTypes::Expression)
    else {
      return None;
    };

    let members = expr_name.members;
    if members.is_empty() {
      expr_name.root_info.call_hooks(parser, hook_call)
    } else {
      expr_name.name.call_hooks(parser, hook_call)
    }
  }
}

impl CallHooks for OptChainExpr {
  fn call_hooks<F, T>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<T>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<T>,
  {
    let Some(MemberExpressionInfo::Expression(expr_name)) = parser
      .get_member_expression_info_from_expr(
        &Expr::OptChain(self.to_owned()),
        AllowedMemberTypes::Expression,
      )
    else {
      return None;
    };

    let members = expr_name.members;
    if members.is_empty() {
      expr_name.root_info.call_hooks(parser, hook_call)
    } else {
      expr_name.name.call_hooks(parser, hook_call)
    }
  }
}

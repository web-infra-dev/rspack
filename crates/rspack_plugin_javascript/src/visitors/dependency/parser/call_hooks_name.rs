use rspack_intern::AtomRef;
use swc_next_ecma_ast::{ChainExpression, Expr, IdentifierReference, MemberExpression};

use super::{AllowedMemberTypes, ExportedVariableInfo, JavascriptParser, MemberExpressionInfo};
use crate::visitors::{ExprRef, scope_info::BindingState};

/// callHooksForName/callHooksForInfo in webpack
/// webpack use HookMap and filter at callHooksForName/callHooksForInfo
/// we need to pass the name to hook to filter in the hook
pub trait CallHooksName {
  fn call_hooks_name<'parser, F, T>(
    &self,
    parser: &mut JavascriptParser<'parser>,
    hook_call: F,
  ) -> Option<T>
  where
    F: Fn(&mut JavascriptParser<'parser>, &str) -> Option<T>;
}

impl CallHooksName for IdentifierReference {
  /// Dispatches hooks through the semantic binding, or the free name when no binding is active.
  fn call_hooks_name<'parser, F, T>(
    &self,
    parser: &mut JavascriptParser<'parser>,
    hook_call: F,
  ) -> Option<T>
  where
    F: Fn(&mut JavascriptParser<'parser>, &str) -> Option<T>,
  {
    let resolution = parser
      .definitions_db
      .semantic_context
      .identifier_resolution(parser.ast, *self);
    if let Some(state) = parser
      .definitions_db
      .resolve_identifier(parser.ast, *self, resolution)
    {
      call_hooks_info(state, parser, hook_call)
    } else {
      let ast = parser.ast.ast;
      hook_call(parser, ast.get_utf8(self.name(ast)))
    }
  }
}

impl<'parser> JavascriptParser<'parser> {
  pub fn call_hooks_name<'key, F, T>(
    &mut self,
    name: impl Into<AtomRef<'key>>,
    hook_call: F,
  ) -> Option<T>
  where
    F: Fn(&mut Self, &str) -> Option<T>,
  {
    let name = name.into();
    if let Some(state) = self.definitions_db.resolve(name) {
      call_hooks_info(state, self, hook_call)
    } else {
      hook_call(self, name.as_str())
    }
  }
}

#[allow(unused_lifetimes)]
impl CallHooksName for ExportedVariableInfo {
  fn call_hooks_name<'parser, F, T>(
    &self,
    parser: &mut JavascriptParser<'parser>,
    hooks_call: F,
  ) -> Option<T>
  where
    F: Fn(&mut JavascriptParser<'parser>, &str) -> Option<T>,
  {
    match self {
      ExportedVariableInfo::Name(n) => parser.call_hooks_name(n, hooks_call),
      ExportedVariableInfo::VariableInfo(v) => call_hooks_info(*v, parser, hooks_call),
    }
  }
}
#[allow(unused_lifetimes)]
impl CallHooksName for MemberExpression {
  fn call_hooks_name<'parser, F, T>(
    &self,
    parser: &mut JavascriptParser<'parser>,
    hook_call: F,
  ) -> Option<T>
  where
    F: Fn(&mut JavascriptParser<'parser>, &str) -> Option<T>,
  {
    let Some(MemberExpressionInfo::Expression(expr_name)) =
      parser.get_member_expression_info(ExprRef::Member(*self), AllowedMemberTypes::Expression)
    else {
      return None;
    };

    let members = expr_name.members;
    if members.is_empty() {
      expr_name.root_info.call_hooks_name(parser, hook_call)
    } else {
      hook_call(parser, &expr_name.name)
    }
  }
}
#[allow(unused_lifetimes)]
impl CallHooksName for ChainExpression {
  fn call_hooks_name<'parser, F, T>(
    &self,
    parser: &mut JavascriptParser<'parser>,
    hook_call: F,
  ) -> Option<T>
  where
    F: Fn(&mut JavascriptParser<'parser>, &str) -> Option<T>,
  {
    let Some(MemberExpressionInfo::Expression(expr_name)) = parser
      .get_member_expression_info_from_expr(
        Expr::ChainExpression(*self),
        AllowedMemberTypes::Expression,
      )
    else {
      return None;
    };

    let members = expr_name.members;
    if members.is_empty() {
      expr_name.root_info.call_hooks_name(parser, hook_call)
    } else {
      hook_call(parser, &expr_name.name)
    }
  }
}

pub(super) fn call_hooks_info<'parser, F, T>(
  id: BindingState,
  parser: &mut JavascriptParser<'parser>,
  hook_call: F,
) -> Option<T>
where
  F: Fn(&mut JavascriptParser<'parser>, &str) -> Option<T>,
{
  if matches!(id, BindingState::Normal(_)) {
    return None;
  }
  let info = parser.definitions_db.expect_get_variable(id);
  let mut next_tag_info = info.tag_info;

  while let Some(tag_info_id) = next_tag_info {
    parser.current_tag_info = Some(tag_info_id);
    let tag_info = parser.definitions_db.expect_get_tag_info(tag_info_id);
    let next = tag_info.next;
    let result = hook_call(parser, tag_info.tag);
    parser.current_tag_info = None;
    if result.is_some() {
      return result;
    }
    next_tag_info = next;
  }

  let info = parser.definitions_db.expect_get_variable(id);
  if let Some(name) = info.name
    && (info.is_free() || info.is_tagged())
  {
    let result = hook_call(parser, &name.clone());
    if result.is_some() {
      return result;
    }
  }

  None
}

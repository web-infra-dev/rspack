use swc_core::atoms::Atom;

use super::{ExportedVariableInfo, JavascriptParser};
use crate::visitors::scope_info::{FreeName, VariableInfoId};

/// callHooksForName/callHooksForInfo in webpack
/// webpack use HookMap and filter at callHooksForName/callHooksForInfo
/// we need to pass the name to hook to filter in the hook
pub trait CallHooksName {
  fn call_hooks_name<F, T>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<T>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<T>;
}

impl CallHooksName for &str {
  fn call_hooks_name<F, T>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<T>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<T>,
  {
    if let Some(id) = parser
      .get_variable_info(self.as_ref())
      .map(|info| info.id())
    {
      // resolved variable info
      call_hooks_info(id, parser, hook_call)
    } else {
      // unresolved variable, for example the global `require` in commonjs.
      hook_call(parser, self)
    }
  }
}

impl CallHooksName for String {
  fn call_hooks_name<'parser, F, T>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<T>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<T>,
  {
    self.as_str().call_hooks_name(parser, hook_call)
  }
}

impl CallHooksName for Atom {
  fn call_hooks_name<'parser, F, T>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<T>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<T>,
  {
    self.as_str().call_hooks_name(parser, hook_call)
  }
}

impl CallHooksName for ExportedVariableInfo {
  fn call_hooks_name<'parser, F, T>(
    &self,
    parser: &mut JavascriptParser,
    hooks_call: F,
  ) -> Option<T>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<T>,
  {
    match self {
      ExportedVariableInfo::Name(n) => n.call_hooks_name(parser, hooks_call),
      ExportedVariableInfo::VariableInfo(v) => call_hooks_info(*v, parser, hooks_call),
    }
  }
}

fn call_hooks_info<F, T>(
  id: VariableInfoId,
  parser: &mut JavascriptParser,
  hook_call: F,
) -> Option<T>
where
  F: Fn(&mut JavascriptParser, &str) -> Option<T>,
{
  let info = parser.definitions_db.expect_get_variable(&id);
  let mut next_tag_info = info.tag_info;

  while let Some(tag_info_id) = next_tag_info {
    parser.current_tag_info = Some(tag_info_id);
    let tag_info = parser.definitions_db.expect_get_tag_info(&tag_info_id);
    let tag = tag_info.tag.to_string();
    let next = tag_info.next;
    let result = hook_call(parser, &tag);
    parser.current_tag_info = None;
    if result.is_some() {
      return result;
    }
    next_tag_info = next;
  }

  let info = parser.definitions_db.expect_get_variable(&id);
  if let Some(FreeName::String(free_name)) = &info.free_name {
    let result = hook_call(parser, &free_name.to_string());
    if result.is_some() {
      return result;
    }
  }
  // should run `defined ? defined() : None` if `free_name` matched FreeName::Tree?

  None
  // maybe we can support `fallback` here
}

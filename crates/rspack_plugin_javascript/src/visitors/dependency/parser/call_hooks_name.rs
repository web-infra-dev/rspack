use swc_core::atoms::Atom;

use super::{ExportedVariableInfo, JavascriptParser};
use crate::visitors::scope_info::{FreeName, VariableInfoId};

/// callHooksForName/callHooksForInfo in webpack
/// webpack use HookMap and filter at callHooksForName/callHooksForInfo
/// we need to pass the name to hook to filter in the hook
pub trait CallHooksName {
  fn call_hooks_name<F>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<bool>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<bool>;
}

impl CallHooksName for &str {
  fn call_hooks_name<F>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<bool>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<bool>,
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
  fn call_hooks_name<F>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<bool>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<bool>,
  {
    self.as_str().call_hooks_name(parser, hook_call)
  }
}

impl CallHooksName for Atom {
  fn call_hooks_name<F>(&self, parser: &mut JavascriptParser, hook_call: F) -> Option<bool>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<bool>,
  {
    self.as_str().call_hooks_name(parser, hook_call)
  }
}

impl CallHooksName for ExportedVariableInfo {
  fn call_hooks_name<F>(&self, parser: &mut JavascriptParser, hooks_call: F) -> Option<bool>
  where
    F: Fn(&mut JavascriptParser, &str) -> Option<bool>,
  {
    match self {
      ExportedVariableInfo::Name(n) => n.call_hooks_name(parser, hooks_call),
      ExportedVariableInfo::VariableInfo(v) => call_hooks_info(*v, parser, hooks_call),
    }
  }
}

fn call_hooks_info<F>(
  id: VariableInfoId,
  parser: &mut JavascriptParser,
  hook_call: F,
) -> Option<bool>
where
  F: Fn(&mut JavascriptParser, &str) -> Option<bool>,
{
  // avoid ownership
  let mut for_name_list = Vec::with_capacity(32);
  let info = parser.definitions_db.expect_get_variable(&id);
  let mut next_tag_info = info.tag_info.as_ref();

  while let Some(tag_info) = next_tag_info {
    for_name_list.push(tag_info.tag.to_string());
    next_tag_info = tag_info.next.as_deref();
  }

  if let Some(FreeName::String(free_name)) = &info.free_name {
    for_name_list.push(free_name.to_string());
  }
  // should run `defined ? defined() : None` if `free_name` matched FreeName::Tree?

  for name in &for_name_list {
    let result = hook_call(parser, name);
    if result.is_some() {
      return result;
    }
  }
  None
  // maybe we can support `fallback` here
}

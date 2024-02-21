use swc_core::atoms::Atom;

use super::{ExportedVariableInfo, JavascriptParser};
use crate::visitors::scope_info::{FreeName, VariableInfo};

/// callHooksForName/callHooksForInfo in webpack
/// webpack use HookMap and filter at callHooksForName/callHooksForInfo
/// we need to pass the name to hook to filter in the hook
pub trait CallHooksName {
  fn call_hooks_name(&self, parser: &mut JavascriptParser) -> Option<String>;
}

impl CallHooksName for &str {
  fn call_hooks_name(&self, parser: &mut JavascriptParser) -> Option<String> {
    if let Some(info) = parser.get_variable_info(self.as_ref()) {
      // resolved variable info
      call_hooks_info(info)
    } else {
      // unresolved variable, for example the global `require` in commonjs.
      Some(self.to_string())
    }
  }
}

impl CallHooksName for String {
  fn call_hooks_name(&self, parser: &mut JavascriptParser) -> Option<String> {
    self.as_str().call_hooks_name(parser)
  }
}

impl CallHooksName for Atom {
  fn call_hooks_name(&self, parser: &mut JavascriptParser) -> Option<String> {
    self.as_str().call_hooks_name(parser)
  }
}

impl CallHooksName for VariableInfo {
  fn call_hooks_name(&self, _parser: &mut JavascriptParser) -> Option<String> {
    call_hooks_info(self)
  }
}

impl CallHooksName for ExportedVariableInfo {
  fn call_hooks_name(&self, parser: &mut JavascriptParser) -> Option<String> {
    match self {
      ExportedVariableInfo::Name(n) => n.call_hooks_name(parser),
      ExportedVariableInfo::VariableInfo(v) => {
        let info = parser.definitions_db.expect_get_variable(v);
        call_hooks_info(info)
      }
    }
  }
}

fn call_hooks_info(info: &VariableInfo) -> Option<String> {
  // TODO: tag_info with hooks
  if let Some(FreeName::String(free_name)) = &info.free_name {
    Some(free_name.to_string())
  } else {
    // should run `defined ? defined() : None`
    None
  }
}

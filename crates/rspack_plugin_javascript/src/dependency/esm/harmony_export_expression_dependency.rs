use rspack_core::{
  CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource,
};

pub const DEFAULT_EXPORT: &str = "__WEBPACK_DEFAULT_EXPORT__";
// pub const NAMESPACE_OBJECT_EXPORT: &'static str = "__WEBPACK_NAMESPACE_OBJECT__";

#[derive(Debug)]
pub struct HarmonyExpressionHeaderDependency {
  pub start: u32,
  pub end: u32,
  pub declaration: bool,
}

impl HarmonyExpressionHeaderDependency {
  pub fn new(start: u32, end: u32, declaration: bool) -> Self {
    Self {
      start,
      end,
      declaration,
    }
  }
}

impl CodeReplaceSourceDependency for HarmonyExpressionHeaderDependency {
  fn apply(
    &self,
    source: &mut CodeReplaceSourceDependencyReplaceSource,
    _code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    if self.declaration {
      source.replace(self.start, self.end, "", None);
    } else {
      source.replace(
        self.start,
        self.end,
        format!("var {DEFAULT_EXPORT} = ").as_str(),
        None,
      );
    }
  }
}

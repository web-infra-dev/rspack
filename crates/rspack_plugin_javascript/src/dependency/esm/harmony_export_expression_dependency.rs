use rspack_core::{CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource};

pub const DEFAULT_EXPORT: &str = "__WEBPACK_DEFAULT_EXPORT__";
// pub const NAMESPACE_OBJECT_EXPORT: &'static str = "__WEBPACK_NAMESPACE_OBJECT__";

#[derive(Debug)]
pub struct HarmonyExpressionHeaderDependency {
  pub start: u32,
  pub end: u32,
  pub declaration: bool,
  pub function: bool,
}

impl HarmonyExpressionHeaderDependency {
  pub fn new(start: u32, end: u32, declaration: bool, function: bool) -> Self {
    Self {
      start,
      end,
      declaration,
      function,
    }
  }
}

impl CodeGeneratableDependency for HarmonyExpressionHeaderDependency {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) {
    if self.declaration {
      // remove export default
      source.replace(self.start, self.end, "", None);
    } else if self.function {
      // hoist function
      source.replace(
        self.start,
        self.end + 8, /* function len */
        format!("function {DEFAULT_EXPORT}").as_str(),
        None,
      );
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

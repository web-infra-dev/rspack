use rspack_core::{CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource};

pub const DEFAULT_EXPORT: &str = "__WEBPACK_DEFAULT_EXPORT__";
// pub const NAMESPACE_OBJECT_EXPORT: &'static str = "__WEBPACK_NAMESPACE_OBJECT__";

#[derive(Debug)]
pub struct AnonymousFunctionRangeInfo {
  pub is_async: bool,
  pub is_generator: bool,
  pub body_start: u32,
  pub first_parmas_start: Option<u32>,
}

#[derive(Debug)]
pub struct HarmonyExpressionHeaderDependency {
  pub start: u32,
  pub end: u32,
  pub declaration: bool,
  pub function: Option<AnonymousFunctionRangeInfo>,
}

impl HarmonyExpressionHeaderDependency {
  pub fn new(
    start: u32,
    end: u32,
    declaration: bool,
    function: Option<AnonymousFunctionRangeInfo>,
  ) -> Self {
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
      source.replace(self.start, self.end, "", None);
    } else if let Some(AnonymousFunctionRangeInfo {
      is_async,
      is_generator,
      body_start,
      first_parmas_start,
    }) = &self.function
    {
      // hoist anonymous function
      let prefix = format!(
        "{}function{} {DEFAULT_EXPORT}",
        if *is_async { "async " } else { "" },
        if *is_generator { "*" } else { "" },
      );
      if let Some(first_parmas_start) = first_parmas_start {
        source.replace(self.start, first_parmas_start - 1, prefix.as_str(), None);
      } else {
        source.replace(
          self.start,
          *body_start,
          format!("{prefix}()").as_str(),
          None,
        );
      }
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

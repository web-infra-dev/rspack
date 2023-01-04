use rspack_error::Result;

pub struct CodeGeneratableContext {}

pub enum CodeGeneratableVisitor {
  JavaScript(
    (
      Vec<swc_core::ecma::visit::AstParentKind>,
      Box<dyn Fn() -> Box<dyn swc_core::ecma::visit::VisitMut>>,
    ),
  ),
  Css(
    (
      Vec<swc_css::visit::AstParentKind>,
      Box<dyn Fn() -> Box<dyn swc_css::visit::VisitMut>>,
    ),
  ),
}

pub struct CodeGeneratableResult {
  pub visitors: Vec<Box<dyn Fn() -> CodeGeneratableVisitor>>,
}

pub trait CodeGeneratable {
  fn generate(
    &self,
    _code_generatable_context: CodeGeneratableContext,
  ) -> Result<CodeGeneratableResult>;
}

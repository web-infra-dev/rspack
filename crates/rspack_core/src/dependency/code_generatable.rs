use crate::{Compilation, CssAstPath, JsAstPath, Module};

pub struct CodeGeneratableContext<'a> {
  compilation: &'a Compilation,
  module: &'a dyn Module,
}

pub trait CodeGeneratable {
  fn generate<'s: 'cx, 'cx>(
    &'s self,
    _code_generatable_context: &'cx CodeGeneratableContext,
  ) -> CodeGeneratableResult<'s>;
}

pub enum CodeGeneratableAstPath {
  JavaScript(JsAstPath),
  Css(CssAstPath),
}

impl From<JsAstPath> for CodeGeneratableAstPath {
  fn from(ast_path: JsAstPath) -> Self {
    Self::JavaScript(ast_path)
  }
}

impl From<CssAstPath> for CodeGeneratableAstPath {
  fn from(ast_path: CssAstPath) -> Self {
    Self::Css(ast_path)
  }
}

pub type CodeGeneratableJavaScriptVisitorBuilder<'v> =
  Box<dyn Fn() -> Box<dyn swc_core::ecma::visit::VisitMut + Send + Sync + 'v>>;

pub type CodeGeneratableCssVisitorBuilder<'v> =
  Box<dyn Fn() -> Box<dyn swc_core::css::visit::VisitMut + Send + Sync + 'v>>;

pub enum CodeGeneratableVisitorBuilder<'b> {
  JavaScript(CodeGeneratableJavaScriptVisitorBuilder<'b>),
  Css(CodeGeneratableCssVisitorBuilder<'b>),
}

pub type CodeGeneratableJavaScriptVisitors<'v> =
  Vec<(JsAstPath, CodeGeneratableJavaScriptVisitorBuilder<'v>)>;

pub type CodeGeneratableCssVisitors<'v> = Vec<(CssAstPath, CodeGeneratableCssVisitorBuilder<'v>)>;

#[derive(Default)]
pub struct CodeGeneratableResult<'b> {
  pub visitors: Vec<(CodeGeneratableAstPath, CodeGeneratableVisitorBuilder<'b>)>,
}

impl<'b> CodeGeneratableResult<'b> {
  /// Convert the code generatable visitors into JavaScript visitors.
  ///
  /// Safety:
  /// It's only safe to be used if all visitors are JavaScript visitors, or it will panic.
  pub fn into_javascript(self) -> CodeGeneratableJavaScriptVisitors<'b> {
    self.visitors.into_iter().map(
      |(ast_path, builder)| {
        if let CodeGeneratableAstPath::JavaScript(ast_path) = ast_path && let CodeGeneratableVisitorBuilder::JavaScript(builder) = builder {
          (ast_path, builder)
        } else {
          panic!("ast_path or builder is not JavaScript")
        }
      },
    ).collect()
  }

  /// Convert the code generatable visitors into Css visitors.
  ///
  /// Safety:
  /// It's only safe to be used if all visitors are Css visitors, or it will panic.
  pub fn into_css(self) -> CodeGeneratableCssVisitors<'b> {
    self.visitors.into_iter().map(
      |(ast_path, builder)| {
        if let CodeGeneratableAstPath::Css(ast_path) = ast_path && let CodeGeneratableVisitorBuilder::Css(builder) = builder {
          (ast_path, builder)
        } else {
          panic!("ast_path or builder is not Css")
        }
      },
    ).collect()
  }
}

/// Creates a single-method visitor that will visit the AST nodes matching the
/// provided path.
///
/// If you pass in `exact`, the visitor will only visit the nodes that match the
/// path exactly. Otherwise, the visitor will visit the closest matching parent
/// node in the path.
///
/// Refer to the [swc_core::ecma::visit::VisitMut] trait for a list of all
/// possible visit methods.
#[macro_export]
macro_rules! create_javascript_visitor {
    (exact $ast_path:expr, $name:ident($arg:ident: &mut $ty:ident) $b:block) => {
        $crate::create_javascript_visitor!(__ $ast_path.to_vec(), $name($arg: &mut $ty) $b)
    };
    ($ast_path:expr, $name:ident($arg:ident: &mut $ty:ident) $b:block) => {
        $crate::create_javascript_visitor!(__ $crate::javascript_path_to(&$ast_path, |n| {
            matches!(n, swc_core::ecma::visit::AstParentKind::$ty(_))
        }), $name($arg: &mut $ty) $b)
    };
    (__ $ast_path:expr, $name:ident($arg:ident: &mut $ty:ident) $b:block) => {{
        struct Visitor<T: Fn(&mut swc_core::ecma::ast::$ty) + Send + Sync> {
            $name: T,
        }

        impl<'a, T: Fn(&mut swc_core::ecma::ast::$ty) + Send + Sync> swc_core::ecma::visit::VisitMut
            for &'a Visitor<T>
        {
            fn $name(&mut self, $arg: &mut swc_core::ecma::ast::$ty) {
                (self.$name)($arg);
            }
        }

        (
            $crate::CodeGeneratableAstPath::from($ast_path.clone()),
            $crate::CodeGeneratableVisitorBuilder::JavaScript(box || {
              box Visitor {
                $name: move |$arg: &mut swc_core::ecma::ast::$ty| $b,
              } as Box<dyn swc_core::ecma::visit::VisitMut + Send + Sync>
            }),
        )
    }};
    (visit_mut_program($arg:ident: &mut Program) $b:block) => {{
        struct Visitor<T: Fn(&mut swc_core::ecma::ast::Program) + Send + Sync> {
            visit_mut_program: T,
        }

        impl<T: Fn(&mut swc_core::ecma::ast::Program) + Send + Sync> $crate::code_gen::VisitorFactory
            for Box<Visitor<T>>
        {
            fn create<'a>(&'a self) -> Box<dyn swc_core::ecma::visit::VisitMut + Send + Sync + 'a> {
                box &**self
            }
        }

        impl<'a, T: Fn(&mut swc_core::ecma::ast::Program) + Send + Sync> swc_core::ecma::visit::VisitMut
            for &'a Visitor<T>
        {
            fn visit_mut_program(&mut self, $arg: &mut swc_core::ecma::ast::Program) {
                (self.visit_mut_program)($arg);
            }
        }

        (
            Vec::new(),
            box box Visitor {
                visit_mut_program: move |$arg: &mut swc_core::ecma::ast::Program| $b,
            } as Box<dyn $crate::code_gen::VisitorFactory>,
        )
    }};
}

pub fn javascript_path_to(
  path: &[swc_core::ecma::visit::AstParentKind],
  f: impl FnMut(&swc_core::ecma::visit::AstParentKind) -> bool,
) -> Vec<swc_core::ecma::visit::AstParentKind> {
  if let Some(pos) = path.iter().rev().position(f) {
    let index = path.len() - pos - 1;
    path[..index].to_vec()
  } else {
    path.to_vec()
  }
}

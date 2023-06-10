/**
 * Some code is modified based on
 * https://github.com/vercel/turbo/blob/a1947f64443fb98e5c3e10bca6ef9eafd278bd21/crates/turbopack-ecmascript/src/code_gen.rs
 * https://github.com/vercel/turbo/blob/a1947f64443fb98e5c3e10bca6ef9eafd278bd21/crates/turbopack-css/src/code_gen.rs
 * MPL-2.0 Licensed
 * Author Alex Kirszenberg, ForsakenHarmony
 * Copyright (c)
 * https://github.com/vercel/turbo/blob/a1947f64443fb98e5c3e10bca6ef9eafd278bd21/LICENSE#L1
 */
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;

use crate::{Compilation, DependencyCategory, Module, ModuleIdentifier, RuntimeGlobals};

pub struct CodeGeneratableContext<'a> {
  pub compilation: &'a Compilation,
  /// Current referenced module
  pub module: &'a dyn Module,
  pub runtime_requirements: &'a mut RuntimeGlobals,
}

pub trait CodeGeneratable {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> Result<CodeGeneratableResult>;
}

pub type JsAstPath = Vec<swc_core::ecma::visit::AstParentKind>;
pub type CssAstPath = Vec<swc_core::css::visit::AstParentKind>;

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

pub type CodeGeneratableJavaScriptVisitor<'v> =
  Box<dyn swc_core::ecma::visit::VisitMut + Send + Sync + 'v>;

pub trait JavaScriptVisitorBuilder {
  fn create(&self) -> CodeGeneratableJavaScriptVisitor;
}

pub type CodeGeneratableCssVisitorBuilder<'v> =
  Box<dyn swc_core::css::visit::VisitMut + Send + Sync + 'v>;

pub trait CssVisitorBuilder {
  fn create(&self) -> CodeGeneratableCssVisitorBuilder;
}

pub enum CodeGeneratableVisitorBuilder {
  JavaScript(Box<dyn JavaScriptVisitorBuilder>),
  Css(Box<dyn CssVisitorBuilder>),
}

pub type CodeGeneratableJavaScriptVisitors = Vec<(JsAstPath, Box<dyn JavaScriptVisitorBuilder>)>;

pub type CodeGeneratableCssVisitors = Vec<(CssAstPath, Box<dyn CssVisitorBuilder>)>;

/// Mapping from the (Module identifier, modified module id, DependencyCategory) to the referencing module identifier
/// This can be used as a workaround to uniquely identify a module from the AST after dependency code generation.
pub type CodeGeneratableDeclMappings =
  HashMap<(ModuleIdentifier, String, DependencyCategory), ModuleIdentifier>;

pub struct CodeGeneratableJavaScriptResult {
  pub visitors: CodeGeneratableJavaScriptVisitors,
  pub decl_mappings: CodeGeneratableDeclMappings,
}

pub struct CodeGeneratableCssResult {
  pub visitors: CodeGeneratableCssVisitors,
  pub decl_mappings: CodeGeneratableDeclMappings,
}

#[derive(Default)]
pub struct CodeGeneratableResult {
  pub visitors: Vec<(CodeGeneratableAstPath, CodeGeneratableVisitorBuilder)>,
  pub decl_mappings: CodeGeneratableDeclMappings,
}

impl CodeGeneratableResult {
  pub fn with_visitors(
    mut self,
    visitors: Vec<(CodeGeneratableAstPath, CodeGeneratableVisitorBuilder)>,
  ) -> Self {
    self.visitors.extend(visitors);
    self
  }

  pub fn with_decl_mappings(mut self, decl_mappings: CodeGeneratableDeclMappings) -> Self {
    self.decl_mappings.extend(decl_mappings);
    self
  }

  pub fn with_visitor(
    mut self,
    visitor: (CodeGeneratableAstPath, CodeGeneratableVisitorBuilder),
  ) -> Self {
    self.visitors.push(visitor);
    self
  }

  pub fn with_decl_mapping(
    mut self,
    key: (ModuleIdentifier, String, DependencyCategory),
    val: ModuleIdentifier,
  ) -> Self {
    self.decl_mappings.insert(key, val);
    self
  }

  /// Convert the code generatable visitors into JavaScript visitors.
  ///
  /// Safety:
  /// It's only safe to be used if all visitors are JavaScript visitors, or it will panic.
  pub fn into_javascript(self) -> CodeGeneratableJavaScriptResult {
    let visitors = self.visitors.into_iter().map(
      |(ast_path, builder)| {
        if let CodeGeneratableAstPath::JavaScript(ast_path) = ast_path && let CodeGeneratableVisitorBuilder::JavaScript(builder) = builder {
          (ast_path, builder)
        } else {
          panic!("Either ast_path or builder is not JavaScript")
        }
      },
    ).collect();

    CodeGeneratableJavaScriptResult {
      visitors,
      decl_mappings: self.decl_mappings,
    }
  }

  /// Convert the code generatable visitors into Css visitors.
  ///
  /// Safety:
  /// It's only safe to be used if all visitors are Css visitors, or it will panic.
  pub fn into_css(self) -> CodeGeneratableCssResult {
    let visitors = self.visitors.into_iter().map(
      |(ast_path, builder)| {
        if let CodeGeneratableAstPath::Css(ast_path) = ast_path && let CodeGeneratableVisitorBuilder::Css(builder) = builder {
          (ast_path, builder)
        } else {
          panic!("Either ast_path or builder is not Css")
        }
      },
    ).collect();

    CodeGeneratableCssResult {
      visitors,
      decl_mappings: self.decl_mappings,
    }
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

        impl<T: Fn(&mut swc_core::ecma::ast::$ty) + Send + Sync> $crate::JavaScriptVisitorBuilder for Box<Visitor<T>> {
          fn create(&self) -> $crate::CodeGeneratableJavaScriptVisitor {
            Box::new(&**self)
          }
        }

        impl<'a, T: Fn(&mut swc_core::ecma::ast::$ty) + Send + Sync> swc_core::ecma::visit::VisitMut
            for &'a Visitor<T>
        {
            fn $name(&mut self, $arg: &mut swc_core::ecma::ast::$ty) {
                (self.$name)($arg);
            }
        }

        (
            $crate::CodeGeneratableAstPath::from($ast_path),
            $crate::CodeGeneratableVisitorBuilder::JavaScript(
              Box::new(Box::new(Visitor {
                $name: move |$arg: &mut swc_core::ecma::ast::$ty| $b,
              })) as Box<dyn $crate::JavaScriptVisitorBuilder>
            ),
        )
    }};
    (visit_mut_program($arg:ident: &mut Program) $b:block) => {{
        struct Visitor<T: Fn(&mut swc_core::ecma::ast::Program) + Send + Sync> {
            visit_mut_program: T,
        }

        impl<T: Fn(&mut swc_core::ecma::ast::Program) + Send + Sync> $crate::JavaScriptVisitorBuilder
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
          $crate::CodeGeneratableAstPath::JavaScript(Vec::new()),
          $crate::CodeGeneratableVisitorBuilder::JavaScript(
              box box Visitor {
                  visit_mut_program: move |$arg: &mut swc_core::ecma::ast::Program| $b,
              } as Box<dyn $crate::JavaScriptVisitorBuilder>,
          ),
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

#[macro_export]
macro_rules! create_css_visitor {
  (visit_mut_stylesheet($arg:ident: &mut Stylesheet) $b:block) => {{
      struct Visitor<T: Fn(&mut swc_core::css::ast::Stylesheet) + Send + Sync> {
          visit_mut_stylesheet: T,
      }

      impl<T: Fn(&mut swc_core::css::ast::Stylesheet) + Send + Sync> $crate::CssVisitorBuilder
          for Box<Visitor<T>>
      {
          fn create<'a>(&'a self) -> Box<dyn swc_core::css::visit::VisitMut + Send + Sync + 'a> {
              Box::new(&**self)
          }
      }

      impl<'a, T: Fn(&mut swc_core::css::ast::Stylesheet) + Send + Sync> swc_core::css::visit::VisitMut
          for &'a Visitor<T>
      {
          fn visit_mut_stylesheet(&mut self, $arg: &mut swc_core::css::ast::Stylesheet) {
              (self.visit_mut_stylesheet)($arg);
          }
      }

      (
          $crate::CodeGeneratableAstPath::Css(Vec::new()),
          $crate::CodeGeneratableVisitorBuilder::Css(
              Box::new(Box::new(Visitor {
                  visit_mut_stylesheet: move |$arg: &mut swc_core::css::ast::Stylesheet| $b,
              })) as Box<dyn $crate::CssVisitorBuilder>
          ),
      )
  }};
  (exact $ast_path:expr, $name:ident($arg:ident: &mut $ty:ident) $b:block) => {
      $crate::create_css_visitor!(__ $ast_path.to_vec(), $name($arg: &mut $ty) $b)
  };
  ($ast_path:expr, $name:ident($arg:ident: &mut $ty:ident) $b:block) => {
      $crate::create_css_visitor!(__ $crate::css_path_to(&$ast_path, |n| {
          matches!(n, swc_core::css::visit::AstParentKind::$ty(_))
      }), $name($arg: &mut $ty) $b)
  };
  (__ $ast_path:expr, $name:ident($arg:ident: &mut $ty:ident) $b:block) => {{
      struct Visitor<T: Fn(&mut swc_core::css::ast::$ty) + Send + Sync> {
          $name: T,
      }

      impl<T: Fn(&mut swc_core::css::ast::$ty) + Send + Sync> $crate::CssVisitorBuilder
          for Box<Visitor<T>>
      {
          fn create<'a>(&'a self) -> Box<dyn swc_core::css::visit::VisitMut + Send + Sync + 'a> {
              Box::new(&**self)
          }
      }

      impl<'a, T: Fn(&mut swc_core::css::ast::$ty) + Send + Sync> swc_core::css::visit::VisitMut
          for &'a Visitor<T>
      {
          fn $name(&mut self, $arg: &mut swc_core::css::ast::$ty) {
              (self.$name)($arg);
          }
      }

      (
          $crate::CodeGeneratableAstPath::from($ast_path),
          $crate::CodeGeneratableVisitorBuilder::Css(
              Box::new(Box::new(Visitor {
                  $name: move |$arg: &mut swc_core::css::ast::$ty| $b,
              })) as Box<dyn $crate::CssVisitorBuilder>
          ),
      )
  }};
}

pub fn css_path_to(
  path: &[swc_core::css::visit::AstParentKind],
  f: impl FnMut(&swc_core::css::visit::AstParentKind) -> bool,
) -> Vec<swc_core::css::visit::AstParentKind> {
  if let Some(pos) = path.iter().rev().position(f) {
    let index = path.len() - pos - 1;
    path[..index].to_vec()
  } else {
    path.to_vec()
  }
}

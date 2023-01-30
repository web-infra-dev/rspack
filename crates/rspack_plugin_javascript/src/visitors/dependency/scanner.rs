use regex::Regex;
use rspack_core::{
  ContextMode, ContextOptions, DependencyCategory, ImportContextDependency, ModuleDependency,
};
use swc_core::common::pass::AstNodePath;
use swc_core::common::{Mark, SyntaxContext};
use swc_core::ecma::ast::{
  CallExpr, Callee, ExportSpecifier, Expr, ExprOrSpread, Lit, MemberProp, MetaPropKind, ModuleDecl,
};
use swc_core::ecma::visit::{AstParentKind, AstParentNodeRef, VisitAstPath, VisitWithPath};

use crate::dependency::{
  CommonJSRequireDependency, EsmDynamicImportDependency, EsmExportDependency, EsmImportDependency,
  ImportMetaModuleHotAcceptDependency, ImportMetaModuleHotDeclineDependency,
  ModuleHotAcceptDependency, ModuleHotDeclineDependency,
};

pub fn as_parent_path(ast_path: &AstNodePath<AstParentNodeRef<'_>>) -> Vec<AstParentKind> {
  ast_path.iter().map(|n| n.kind()).collect()
}

pub struct DependencyScanner {
  pub unresolved_ctxt: SyntaxContext,
  pub dependencies: Vec<Box<dyn ModuleDependency>>,
  // pub dyn_dependencies: HashSet<DynImportDesc>,
}

impl DependencyScanner {
  fn add_dependency(&mut self, dependency: Box<dyn ModuleDependency>) {
    self.dependencies.push(dependency);
  }

  fn add_import(&mut self, module_decl: &ModuleDecl, ast_path: &AstNodePath<AstParentNodeRef<'_>>) {
    if let ModuleDecl::Import(import_decl) = module_decl {
      let source = import_decl.src.value.clone();
      self.add_dependency(box EsmImportDependency::new(
        source,
        Some(import_decl.span.into()),
        as_parent_path(ast_path),
      ));
    }
  }
  fn add_require(&mut self, call_expr: &CallExpr, ast_path: &AstNodePath<AstParentNodeRef<'_>>) {
    if let Callee::Expr(expr) = &call_expr.callee {
      if let Expr::Ident(ident) = &**expr {
        if "require".eq(&ident.sym) && ident.span.ctxt == self.unresolved_ctxt {
          {
            if call_expr.args.len() != 1 {
              return;
            }
            let src = match call_expr.args.first().expect("TODO:") {
              ExprOrSpread { spread: None, expr } => match &**expr {
                Expr::Lit(Lit::Str(s)) => s,
                _ => return,
              },
              _ => return,
            };
            let source = src.value.clone();
            self.add_dependency(box CommonJSRequireDependency::new(
              source,
              Some(call_expr.span.into()),
              as_parent_path(ast_path),
            ));
            // self.add_dependency(source.clone(), ResolveKind::Require, call_expr.span);
          }
        }
      }
    }
  }
  fn add_dynamic_import(&mut self, node: &CallExpr, ast_path: &AstNodePath<AstParentNodeRef<'_>>) {
    if let Callee::Import(_) = node.callee {
      if let Some(dyn_imported) = node.args.get(0) {
        if dyn_imported.spread.is_none() {
          if let Expr::Lit(Lit::Str(imported)) = dyn_imported.expr.as_ref() {
            self.add_dependency(box EsmDynamicImportDependency::new(
              imported.value.clone(),
              Some(node.span.into()),
              as_parent_path(ast_path),
            ));
          }
          if let Expr::Tpl(tpl) = dyn_imported.expr.as_ref() {
            let prefix_raw = tpl
              .quasis
              .first()
              .expect("should have one quasis")
              .raw
              .to_string();
            let post_raw = if tpl.quasis.len() > 1 {
              tpl
                .quasis
                .last()
                .expect("should have last quasis")
                .raw
                .to_string()
            } else {
              String::new()
            };
            let (context, prefix) = split_context_from_prefix(&prefix_raw);
            let inner_reg = tpl
              .quasis
              .iter()
              .skip(1)
              .map(|_| ".*")
              .collect::<Vec<&str>>()
              .join("");
            let reg = format!("^{prefix}{inner_reg}{post_raw}$");
            self.add_dependency(box ImportContextDependency {
              options: ContextOptions {
                mode: ContextMode::Lazy, // lazy by default
                recursive: false,
                reg_exp: Regex::new(&reg).expect("reg failed"),
                include: None,
                exclude: None,
                category: DependencyCategory::Esm,
                request: context.to_string(),
              },
              ast_path: as_parent_path(ast_path),
              parent_module_identifier: None,
            });
          }
        }
      }
    }
  }

  fn add_module_hot(&mut self, node: &CallExpr, ast_path: &AstNodePath<AstParentNodeRef<'_>>) {
    let is_module_hot = is_module_hot_accept_call(node);
    let is_module_decline = is_module_hot_decline_call(node);
    let is_import_meta_hot_accept = is_import_meta_hot_accept_call(node);
    let is_import_meta_hot_decline = is_import_meta_hot_decline_call(node);

    if !is_module_hot
      && !is_module_decline
      && !is_import_meta_hot_accept
      && !is_import_meta_hot_decline
    {
      return;
    }

    if let Some(Lit::Str(str)) = node
      .args
      .get(0)
      .and_then(|first_arg| first_arg.expr.as_lit())
    {
      if is_module_hot {
        // module.hot.accept(dependency_id, callback)
        self.add_dependency(box ModuleHotAcceptDependency::new(
          str.value.clone(),
          Some(node.span.into()),
          as_parent_path(ast_path),
        ));
      } else if is_module_decline {
        self.add_dependency(box ModuleHotDeclineDependency::new(
          str.value.clone(),
          Some(node.span.into()),
          as_parent_path(ast_path),
        ));
      } else if is_import_meta_hot_accept {
        self.add_dependency(box ImportMetaModuleHotAcceptDependency::new(
          str.value.clone(),
          Some(node.span.into()),
          as_parent_path(ast_path),
        ));
      } else if is_import_meta_hot_decline {
        self.add_dependency(box ImportMetaModuleHotDeclineDependency::new(
          str.value.clone(),
          Some(node.span.into()),
          as_parent_path(ast_path),
        ));
      }
    }
  }

  fn add_export(
    &mut self,
    module_decl: &ModuleDecl,
    ast_path: &AstNodePath<AstParentNodeRef<'_>>,
  ) -> Result<(), anyhow::Error> {
    match module_decl {
      ModuleDecl::ExportNamed(node) => {
        node.specifiers.iter().for_each(|specifier| {
          match specifier {
            ExportSpecifier::Named(_s) => {
              if let Some(source_node) = &node.src {
                // export { name } from './other'
                // TODO: this should ignore from code generation or use a new dependency instead
                self.add_dependency(box EsmExportDependency::new(
                  source_node.value.clone(),
                  Some(node.span.into()),
                  as_parent_path(ast_path),
                ));
              }
            }
            ExportSpecifier::Namespace(_s) => {
              // export * as name from './other'
              let source = node
                .src
                .as_ref()
                .map(|str| str.value.clone())
                .expect("TODO:");
              // TODO: this should ignore from code generation or use a new dependency instead
              self.add_dependency(box EsmExportDependency::new(
                source,
                Some(node.span.into()),
                as_parent_path(ast_path),
              ));
            }
            ExportSpecifier::Default(_) => {
              // export v from 'mod';
              // Rollup doesn't support it.
            }
          };
        });
      }
      ModuleDecl::ExportAll(node) => {
        // export * from './other'
        // TODO: this should ignore from code generation or use a new dependency instead
        self.add_dependency(box EsmExportDependency::new(
          node.src.value.clone(),
          Some(node.span.into()),
          as_parent_path(ast_path),
        ));
      }
      _ => {}
    }
    Ok(())
  }
}

impl VisitAstPath for DependencyScanner {
  fn visit_module_decl<'ast: 'r, 'r>(
    &mut self,
    node: &'ast ModuleDecl,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    self.add_import(node, &*ast_path);
    if let Err(e) = self.add_export(node, &*ast_path) {
      eprintln!("{e}");
    }
    node.visit_children_with_path(self, ast_path);
  }
  fn visit_call_expr<'ast: 'r, 'r>(
    &mut self,
    node: &'ast CallExpr,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    self.add_module_hot(node, &*ast_path);
    self.add_dynamic_import(node, &*ast_path);
    self.add_require(node, &*ast_path);
    node.visit_children_with_path(self, ast_path);
  }
}

impl DependencyScanner {
  pub fn new(unresolved_mark: Mark) -> Self {
    Self {
      unresolved_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
      dependencies: Default::default(),
    }
  }
}

#[inline]
fn split_context_from_prefix(prefix: &str) -> (&str, &str) {
  if let Some(idx) = prefix.rfind('/') {
    (&prefix[..idx], &prefix[idx + 1..])
  } else {
    (".", prefix)
  }
}

#[test]
fn test_dependency_scanner() {
  // TODO: temporarily disabled for new dependency impl

  // use crate::ast::parse_js_code;
  // use rspack_core::{ErrorSpan, ModuleType};
  // use swc_core::ecma::visit::{VisitMutWith, VisitWith};

  // let code = r#"
  // const a = require('a');
  // exports.b = require('b');
  // module.hot.accept('e', () => {})
  // import f from 'g';
  // import * as h from 'i';
  // import { j } from 'k';
  // import { default as l } from 'm';
  // "#;
  // let mut ast = parse_js_code(code.to_string(), &ModuleType::Js).expect("TODO:");
  // let dependencies = swc_core::common::GLOBALS.set(&Default::default(), || {
  //   let unresolved_mark = Mark::new();
  //   let mut resolver =
  //     swc_core::ecma::transforms::base::resolver(unresolved_mark, Mark::new(), false);
  //   ast.visit_mut_with(&mut resolver);
  //   let mut scanner = DependencyScanner::new(unresolved_mark);
  //   ast.visit_with(&mut scanner);
  //   scanner.dependencies
  // });
  // let mut iter = dependencies.into_iter();
  // assert_eq!(
  //   iter.next().expect("TODO:"),
  //   ModuleDependency {
  //     specifier: "a".to_string(),
  //     kind: ResolveKind::Require,
  //     span: Some(ErrorSpan { start: 13, end: 25 },),
  //   }
  // );
  // assert_eq!(
  //   iter.next().expect("TODO:"),
  //   ModuleDependency {
  //     specifier: "b".to_string(),
  //     kind: ResolveKind::Require,
  //     span: Some(ErrorSpan { start: 41, end: 53 },),
  //   },
  // );
  // assert_eq!(
  //   iter.next().expect("TODO:"),
  //   ModuleDependency {
  //     specifier: "e".to_string(),
  //     kind: ResolveKind::ModuleHotAccept,
  //     span: Some(ErrorSpan { start: 57, end: 89 },),
  //   },
  // );
  // assert_eq!(
  //   iter.next().expect("TODO:"),
  //   ModuleDependency {
  //     specifier: "g".to_string(),
  //     kind: ResolveKind::Import,
  //     span: Some(ErrorSpan {
  //       start: 92,
  //       end: 110,
  //     },),
  //   },
  // );
  // assert_eq!(
  //   iter.next().expect("TODO:"),
  //   ModuleDependency {
  //     specifier: "i".to_string(),
  //     kind: ResolveKind::Import,
  //     span: Some(ErrorSpan {
  //       start: 113,
  //       end: 136,
  //     },),
  //   },
  // );
  // assert_eq!(
  //   iter.next().expect("TODO:"),
  //   ModuleDependency {
  //     specifier: "k".to_string(),
  //     kind: ResolveKind::Import,
  //     span: Some(ErrorSpan {
  //       start: 139,
  //       end: 161,
  //     },),
  //   },
  // );
  // assert_eq!(
  //   iter.next().expect("TODO:"),
  //   ModuleDependency {
  //     specifier: "m".to_string(),
  //     kind: ResolveKind::Import,
  //     span: Some(ErrorSpan {
  //       start: 164,
  //       end: 197,
  //     },),
  //   },
  // )
}

fn match_member_expr(mut expr: &Expr, value: &str) -> bool {
  let mut parts: Vec<&str> = value.split('.').collect();
  parts.reverse();
  let last = parts.pop().expect("should have a last str");
  for part in parts {
    if let Expr::Member(member_expr) = expr {
      if let MemberProp::Ident(ident) = &member_expr.prop {
        if ident.sym.eq(part) {
          expr = &member_expr.obj;
          continue;
        }
      }
    }
    return false;
  }
  matches!(&expr, Expr::Ident(ident) if ident.sym.eq(last))
}

#[inline]
fn is_hmr_api_call(node: &CallExpr, value: &str) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| match_member_expr(expr, value))
    .unwrap_or_default()
}

pub fn is_module_hot_accept_call(node: &CallExpr) -> bool {
  is_hmr_api_call(node, "module.hot.accept")
}

pub fn is_module_hot_decline_call(node: &CallExpr) -> bool {
  is_hmr_api_call(node, "module.hot.decline")
}

fn match_import_meta_member_expr(mut expr: &Expr, value: &str) -> bool {
  let mut parts: Vec<&str> = value.split('.').collect();
  parts.reverse();
  // pop import.meta
  parts.pop();
  parts.pop();
  for part in parts {
    if let Expr::Member(member_expr) = expr {
      if let MemberProp::Ident(ident) = &member_expr.prop {
        if ident.sym.eq(part) {
          expr = &member_expr.obj;
          continue;
        }
      }
    }
    return false;
  }
  matches!(&expr, Expr::MetaProp(meta) if meta.kind == MetaPropKind::ImportMeta)
}

fn is_hmr_import_meta_api_call(node: &CallExpr, value: &str) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| match_import_meta_member_expr(expr, value))
    .unwrap_or_default()
}

pub fn is_import_meta_hot_accept_call(node: &CallExpr) -> bool {
  is_hmr_import_meta_api_call(node, "import.meta.webpackHot.accept")
}

pub fn is_import_meta_hot_decline_call(node: &CallExpr) -> bool {
  is_hmr_import_meta_api_call(node, "import.meta.webpackHot.decline")
}

#[test]
fn test() {
  use swc_core::common::DUMMY_SP;
  use swc_core::ecma::ast::{Ident, MemberExpr, MetaPropExpr};
  use swc_core::ecma::utils::member_expr;
  use swc_core::ecma::utils::ExprFactory;
  let expr = *member_expr!(DUMMY_SP, module.hot.accept);
  assert!(match_member_expr(&expr, "module.hot.accept"));
  assert!(is_module_hot_accept_call(&CallExpr {
    span: DUMMY_SP,
    callee: expr.as_callee(),
    args: vec![],
    type_args: None
  }));

  let import_meta_expr = Expr::Member(MemberExpr {
    span: DUMMY_SP,
    obj: Box::new(Expr::Member(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(Expr::MetaProp(MetaPropExpr {
        span: DUMMY_SP,
        kind: MetaPropKind::ImportMeta,
      })),
      prop: MemberProp::Ident(Ident::new("webpackHot".into(), DUMMY_SP)),
    })),
    prop: MemberProp::Ident(Ident::new("accept".into(), DUMMY_SP)),
  });
  assert!(match_import_meta_member_expr(
    &import_meta_expr,
    "import.meta.webpackHot.accept"
  ));
  assert!(is_import_meta_hot_accept_call(&CallExpr {
    span: DUMMY_SP,
    callee: import_meta_expr.as_callee(),
    args: vec![],
    type_args: None
  }));
}

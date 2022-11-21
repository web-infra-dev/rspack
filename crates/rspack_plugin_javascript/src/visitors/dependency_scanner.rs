use linked_hash_set::LinkedHashSet;
use rspack_core::{ModuleDependency, ResolveKind};
use swc_atoms::JsWord;
use swc_common::{Mark, Span};
use swc_ecma_ast::{CallExpr, Callee, ExportSpecifier, Expr, ExprOrSpread, Lit, ModuleDecl};
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};

pub struct DependencyScanner {
  pub unresolved_mark: Mark,
  pub dependencies: LinkedHashSet<ModuleDependency>,
  // pub dyn_dependencies: HashSet<DynImportDesc>,
}

impl DependencyScanner {
  fn add_dependency(&mut self, specifier: JsWord, kind: ResolveKind, span: Span) {
    self.dependencies.insert_if_absent(ModuleDependency {
      specifier: specifier.to_string(),
      kind,
      span: Some(span.into()),
    });
  }

  fn add_import(&mut self, module_decl: &ModuleDecl) {
    if let ModuleDecl::Import(import_decl) = module_decl {
      let source = import_decl.src.value.clone();
      self.add_dependency(source, ResolveKind::Import, import_decl.span);
    }
  }
  fn add_require(&mut self, call_expr: &CallExpr) {
    if let Callee::Expr(expr) = &call_expr.callee {
      if let Expr::Ident(ident) = &**expr {
        if "require".eq(&ident.sym) {
          {
            if call_expr.args.len() != 1 {
              return;
            }
            let src = match call_expr.args.first().unwrap() {
              ExprOrSpread { spread: None, expr } => match &**expr {
                Expr::Lit(Lit::Str(s)) => s,
                _ => return,
              },
              _ => return,
            };
            let source = &src.value;
            self.add_dependency(source.clone(), ResolveKind::Require, call_expr.span);
          }
        }
      }
    }
  }
  fn add_dynamic_import(&mut self, node: &CallExpr) {
    if let Callee::Import(_) = node.callee {
      if let Some(dyn_imported) = node.args.get(0) {
        if dyn_imported.spread.is_none() {
          if let Expr::Lit(Lit::Str(imported)) = dyn_imported.expr.as_ref() {
            self.add_dependency(
              imported.value.clone(),
              ResolveKind::DynamicImport,
              node.span,
            );
          }
        }
      }
    }
  }

  fn add_module_hot(&mut self, node: &CallExpr) {
    if !is_module_hot_accept_call(node) {
      return;
    }

    // module.hot.accept(dependency_id, callback)
    if let Some(Lit::Str(str)) = node
      .args
      .get(0)
      .and_then(|first_arg| first_arg.expr.as_lit())
    {
      self.add_dependency(str.value.clone(), ResolveKind::ModuleHotAccept, node.span)
    }
  }

  fn add_export(&mut self, module_decl: &ModuleDecl) -> Result<(), anyhow::Error> {
    match module_decl {
      ModuleDecl::ExportNamed(node) => {
        node.specifiers.iter().for_each(|specifier| {
          match specifier {
            ExportSpecifier::Named(_s) => {
              if let Some(source_node) = &node.src {
                // export { name } from './other'
                let source = source_node.value.clone();
                self.add_dependency(source, ResolveKind::Import, node.span);
              }
            }
            ExportSpecifier::Namespace(_s) => {
              // export * as name from './other'
              let source = node.src.as_ref().map(|str| str.value.clone()).unwrap();
              self.add_dependency(source, ResolveKind::Import, node.span);
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
        self.add_dependency(node.src.value.clone(), ResolveKind::Import, node.span);
      }
      _ => {}
    }
    Ok(())
  }
}

impl Visit for DependencyScanner {
  noop_visit_type!();

  fn visit_module_decl(&mut self, node: &ModuleDecl) {
    self.add_import(node);
    if let Err(e) = self.add_export(node) {
      eprintln!("{}", e);
    }
    node.visit_children_with(self);
  }
  fn visit_call_expr(&mut self, node: &CallExpr) {
    self.add_module_hot(node);
    self.add_dynamic_import(node);
    self.add_require(node);
    node.visit_children_with(self);
  }
}

impl DependencyScanner {
  pub fn new(unresolved_mark: Mark) -> Self {
    Self {
      unresolved_mark,
      dependencies: Default::default(),
    }
  }
}

#[test]
fn test_dependency_scanner() {
  use crate::ast::parse_js_code;
  use rspack_core::{ErrorSpan, ModuleType};
  use swc_ecma_visit::VisitWith;

  let code = r#"
  const a = require('a');
  exports.b = require('b');
  module.hot.accept('e', () => {})
  import f from 'g';
  import * as h from 'i';
  import { j } from 'k';
  import { default as l } from 'm';
  "#;
  let ast = parse_js_code(code.to_string(), &ModuleType::Js).unwrap();
  let mut scanner = DependencyScanner::default();
  ast.visit_with(&mut scanner);
  let mut iter = scanner.dependencies.into_iter();
  assert_eq!(
    iter.next().unwrap(),
    ModuleDependency {
      specifier: "a".to_string(),
      kind: ResolveKind::Require,
      span: Some(ErrorSpan { start: 13, end: 25 },),
    }
  );
  assert_eq!(
    iter.next().unwrap(),
    ModuleDependency {
      specifier: "b".to_string(),
      kind: ResolveKind::Require,
      span: Some(ErrorSpan { start: 41, end: 53 },),
    },
  );
  assert_eq!(
    iter.next().unwrap(),
    ModuleDependency {
      specifier: "e".to_string(),
      kind: ResolveKind::ModuleHotAccept,
      span: Some(ErrorSpan { start: 57, end: 89 },),
    },
  );
  assert_eq!(
    iter.next().unwrap(),
    ModuleDependency {
      specifier: "g".to_string(),
      kind: ResolveKind::Import,
      span: Some(ErrorSpan {
        start: 92,
        end: 110,
      },),
    },
  );
  assert_eq!(
    iter.next().unwrap(),
    ModuleDependency {
      specifier: "i".to_string(),
      kind: ResolveKind::Import,
      span: Some(ErrorSpan {
        start: 113,
        end: 136,
      },),
    },
  );
  assert_eq!(
    iter.next().unwrap(),
    ModuleDependency {
      specifier: "k".to_string(),
      kind: ResolveKind::Import,
      span: Some(ErrorSpan {
        start: 139,
        end: 161,
      },),
    },
  );
  assert_eq!(
    iter.next().unwrap(),
    ModuleDependency {
      specifier: "m".to_string(),
      kind: ResolveKind::Import,
      span: Some(ErrorSpan {
        start: 164,
        end: 197,
      },),
    },
  )
}

pub fn is_module_hot_accept_call(node: &CallExpr) -> bool {
  node
    .callee
    .as_expr()
    .and_then(|expr| {
      // let target = swc_ecma_utils::member_expr!(DUMMY_SP, module.hot.accept);
      // target.eq_ignore_span(expr)

      expr.as_member()
    })
    .and_then(|member| {
      // TODO: `swc_visitor::resolver` had make `target.eq_ignore_span(expr)`
      //  return false.
      // TODO: Delete following code when find a good way
      member
        .prop
        .as_ident()
        .and_then(|ident| "accept".eq(&ident.sym).then_some(&member.obj))
    })
    .and_then(|obj| obj.as_member())
    .and_then(|member| {
      member
        .prop
        .as_ident()
        .and_then(|ident| "hot".eq(&ident.sym).then_some(&member.obj))
    })
    .and_then(|obj| obj.as_ident())
    .map(|ident| "module".eq(&ident.sym))
    .unwrap_or_default()
}

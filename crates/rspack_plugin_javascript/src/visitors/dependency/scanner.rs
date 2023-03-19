use rspack_core::{
  runtime_globals, CommonJsRequireContextDependency, CompilerOptions, ConstDependency, ContextMode,
  ContextOptions, Dependency, DependencyCategory, ImportContextDependency, ModuleDependency,
  RequireContextDependency, ResourceData,
};
use rspack_regex::RspackRegex;
use sugar_path::SugarPath;
use swc_core::common::{pass::AstNodePath, Mark, SyntaxContext};
use swc_core::ecma::ast::{
  BinExpr, BinaryOp, CallExpr, Callee, ExportSpecifier, Expr, Lit, MemberProp, MetaPropKind,
  ModuleDecl, Tpl,
};
use swc_core::ecma::utils::{quote_ident, quote_str};
use swc_core::ecma::visit::{AstParentKind, AstParentNodeRef, VisitAstPath, VisitWithPath};
use swc_core::quote;

use crate::dependency::{
  CommonJSRequireDependency, EsmDynamicImportDependency, EsmExportDependency, EsmImportDependency,
  ImportMetaModuleHotAcceptDependency, ImportMetaModuleHotDeclineDependency,
  ModuleHotAcceptDependency, ModuleHotDeclineDependency,
};

pub const WEBPACK_HASH: &str = "__webpack_hash__";
pub const WEBPACK_PUBLIC_PATH: &str = "__webpack_public_path__";
pub const DIR_NAME: &str = "__dirname";
pub const FILE_NAME: &str = "__filename";
pub const WEBPACK_MODULES: &str = "__webpack_modules__";
pub const WEBPACK_RESOURCE_QUERY: &str = "__resourceQuery";
pub const GLOBAL: &str = "global";

pub fn as_parent_path(ast_path: &AstNodePath<AstParentNodeRef<'_>>) -> Vec<AstParentKind> {
  ast_path.iter().map(|n| n.kind()).collect()
}

pub struct DependencyScanner<'a> {
  pub unresolved_ctxt: SyntaxContext,
  pub dependencies: Vec<Box<dyn ModuleDependency>>,
  pub presentational_dependencies: Vec<Box<dyn Dependency>>,
  pub compiler_options: &'a CompilerOptions,
  pub resource_data: &'a ResourceData,
}

impl DependencyScanner<'_> {
  fn add_dependency(&mut self, dependency: Box<dyn ModuleDependency>) {
    self.dependencies.push(dependency);
  }

  fn add_presentational_dependency(&mut self, dependency: Box<dyn Dependency>) {
    self.presentational_dependencies.push(dependency);
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
            if let Some(expr) = call_expr.args.get(0) {
              if expr.spread.is_none() {
                if let Expr::Lit(Lit::Str(s)) = expr.expr.as_ref() {
                  self.add_dependency(box CommonJSRequireDependency::new(
                    s.value.clone(),
                    Some(call_expr.span.into()),
                    as_parent_path(ast_path),
                  ));
                }

                if let Some((context, reg)) = scanner_context_module(expr.expr.as_ref()) {
                  self.add_dependency(box CommonJsRequireContextDependency::new(
                    ContextOptions {
                      mode: ContextMode::Sync,
                      recursive: true,
                      reg_exp: RspackRegex::new(&reg).expect("reg failed"),
                      reg_str: reg,
                      include: None,
                      exclude: None,
                      category: DependencyCategory::CommonJS,
                      request: context,
                    },
                    Some(call_expr.span.into()),
                    as_parent_path(ast_path),
                  ));
                }
              }
            }
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
          if let Some((context, reg)) = scanner_context_module(dyn_imported.expr.as_ref()) {
            self.add_dependency(box ImportContextDependency::new(
              ContextOptions {
                mode: ContextMode::Lazy,
                recursive: true,
                reg_exp: RspackRegex::new(&reg).expect("reg failed"),
                reg_str: reg,
                include: None,
                exclude: None,
                category: DependencyCategory::Esm,
                request: context,
              },
              Some(node.span.into()),
              as_parent_path(ast_path),
            ));
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

  fn scan_require_context(
    &mut self,
    node: &CallExpr,
    ast_path: &AstNodePath<AstParentNodeRef<'_>>,
  ) {
    if is_require_context_call(node) && !node.args.is_empty() {
      if let Some(Lit::Str(str)) = node.args.get(0).and_then(|x| x.expr.as_lit()) {
        let recursive =
          if let Some(Lit::Bool(bool)) = node.args.get(1).and_then(|x| x.expr.as_lit()) {
            bool.value
          } else {
            true
          };

        let (reg_exp, reg_str) =
          if let Some(Lit::Regex(regex)) = node.args.get(2).and_then(|x| x.expr.as_lit()) {
            (
              RspackRegex::try_from(regex).expect("reg failed"),
              format!("{}|{}", regex.exp, regex.flags),
            )
          } else {
            (
              RspackRegex::new(r"^\.\/.*$").expect("reg failed"),
              r"^\.\/.*$".to_string(),
            )
          };

        let mode = if let Some(Lit::Str(str)) = node.args.get(3).and_then(|x| x.expr.as_lit()) {
          match str.value.to_string().as_str() {
            "sync" => ContextMode::Sync,
            "eager" => ContextMode::Eager,
            "weak" => ContextMode::Weak,
            "lazy" => ContextMode::Lazy,
            "lazy-once" => ContextMode::LazyOnce,
            // TODO should give warning
            _ => unreachable!("unknown context mode"),
          }
        } else {
          ContextMode::Sync
        };
        self.add_dependency(box RequireContextDependency::new(
          ContextOptions {
            mode,
            recursive,
            reg_exp,
            reg_str,
            include: None,
            exclude: None,
            category: DependencyCategory::CommonJS,
            request: str.value.to_string(),
          },
          Some(node.span.into()),
          as_parent_path(ast_path),
        ));
      }
    }
  }
}

impl VisitAstPath for DependencyScanner<'_> {
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
    self.scan_require_context(node, &*ast_path);
    node.visit_children_with_path(self, ast_path);
  }

  fn visit_expr<'ast: 'r, 'r>(
    &mut self,
    expr: &'ast Expr,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    if let Expr::Ident(ident) = expr {
      if ident.span.ctxt == self.unresolved_ctxt {
        match ident.sym.as_ref() as &str {
          WEBPACK_HASH => {
            self.add_presentational_dependency(box ConstDependency::new(
              quote!(
                "$name()" as Expr,
                name = quote_ident!(runtime_globals::GET_FULL_HASH)
              ),
              Some(runtime_globals::GET_FULL_HASH),
              as_parent_path(ast_path),
            ));
          }
          WEBPACK_PUBLIC_PATH => {
            self.add_presentational_dependency(box ConstDependency::new(
              Expr::Ident(quote_ident!(runtime_globals::PUBLIC_PATH)),
              Some(runtime_globals::PUBLIC_PATH),
              as_parent_path(ast_path),
            ));
          }
          WEBPACK_MODULES => {
            self.add_presentational_dependency(box ConstDependency::new(
              Expr::Ident(quote_ident!(runtime_globals::MODULE_FACTORIES)),
              Some(runtime_globals::MODULE_FACTORIES),
              as_parent_path(ast_path),
            ));
          }
          WEBPACK_RESOURCE_QUERY => {
            if let Some(resource_query) = &self.resource_data.resource_query {
              self.add_presentational_dependency(box ConstDependency::new(
                Expr::Lit(Lit::Str(quote_str!(resource_query.to_owned()))),
                None,
                as_parent_path(ast_path),
              ));
            }
          }
          DIR_NAME => {
            let dirname = match self.compiler_options.node.dirname.as_str() {
              "mock" => Some("/".to_string()),
              "warn-mock" => Some("/".to_string()),
              "true" => Some(
                self
                  .resource_data
                  .resource_path
                  .parent()
                  .expect("TODO:")
                  .relative(self.compiler_options.context.as_ref())
                  .to_string_lossy()
                  .to_string(),
              ),
              _ => None,
            };
            if let Some(dirname) = dirname {
              self.add_presentational_dependency(box ConstDependency::new(
                Expr::Lit(Lit::Str(quote_str!(dirname))),
                None,
                as_parent_path(ast_path),
              ));
            }
          }
          FILE_NAME => {
            let filename = match self.compiler_options.node.filename.as_str() {
              "mock" => Some("/index.js".to_string()),
              "warn-mock" => Some("/index.js".to_string()),
              "true" => Some(
                self
                  .resource_data
                  .resource_path
                  .relative(self.compiler_options.context.as_ref())
                  .to_string_lossy()
                  .to_string(),
              ),
              _ => None,
            };
            if let Some(filename) = filename {
              self.add_presentational_dependency(box ConstDependency::new(
                Expr::Lit(Lit::Str(quote_str!(filename))),
                None,
                as_parent_path(ast_path),
              ));
            }
          }
          GLOBAL => {
            if matches!(self.compiler_options.node.global.as_str(), "true" | "warn") {
              self.add_presentational_dependency(box ConstDependency::new(
                Expr::Ident(quote_ident!(runtime_globals::GLOBAL)),
                Some(runtime_globals::GLOBAL),
                as_parent_path(ast_path),
              ));
            }
          }
          _ => {}
        }
      }
    }
    expr.visit_children_with_path(self, ast_path);
  }
}

impl<'a> DependencyScanner<'a> {
  pub fn new(
    unresolved_mark: Mark,
    resource_data: &'a ResourceData,
    compiler_options: &'a CompilerOptions,
  ) -> Self {
    Self {
      unresolved_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
      dependencies: Default::default(),
      presentational_dependencies: Default::default(),
      compiler_options,
      resource_data,
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

fn scanner_context_module(expr: &Expr) -> Option<(String, String)> {
  match expr {
    Expr::Tpl(tpl) => Some(scan_context_module_tpl(tpl)),
    Expr::Bin(bin) => scan_context_module_bin(bin),
    Expr::Call(call) => scan_context_module_concat_call(call),
    _ => None,
  }
}

// require(`./${a}.js`)
fn scan_context_module_tpl(tpl: &Tpl) -> (String, String) {
  let prefix_raw = tpl
    .quasis
    .first()
    .expect("should have one quasis")
    .raw
    .to_string();
  let postfix_raw = if tpl.quasis.len() > 1 {
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
    .skip(tpl.quasis.len())
    .skip(1)
    .map(|s| s.raw.to_string() + ".*")
    .collect::<Vec<String>>()
    .join("");
  let reg = format!("^{prefix}.*{inner_reg}{postfix_raw}$");
  (context.to_string(), reg)
}

// require("./" + a + ".js")
fn scan_context_module_bin(bin: &BinExpr) -> Option<(String, String)> {
  if !is_add_op_bin_expr(bin) {
    return None;
  }
  let prefix_raw = if let Some(prefix) = find_expr_prefix_string(&bin.left) {
    prefix
  } else {
    "".to_string()
  };
  let postfix_raw = if let Some(postfix) = find_expr_prefix_string(&bin.right) {
    postfix
  } else {
    "".to_string()
  };

  if prefix_raw.is_empty() && postfix_raw.is_empty() {
    return None;
  }

  let (context, prefix) = split_context_from_prefix(&prefix_raw);
  let reg = format!("^{prefix}.*{postfix_raw}$");

  Some((context.to_string(), reg))
}

fn find_expr_prefix_string(expr: &Expr) -> Option<String> {
  match &expr {
    Expr::Lit(Lit::Str(str)) => Some(str.value.to_string()),
    Expr::Lit(Lit::Num(num)) => Some(num.value.to_string()),
    Expr::Bin(bin) => find_expr_prefix_string(&bin.left),
    _ => None,
  }
}

fn is_add_op_bin_expr(bin: &BinExpr) -> bool {
  if !matches!(&bin.op, BinaryOp::Add) {
    return false;
  }
  match &bin.left {
    box Expr::Bin(bin) => is_add_op_bin_expr(bin),
    _ => true,
  }
}

// require("./".concat(a, ".js"))
// babel/swc will transform template literal to string concat, so we need to handle this case
// see https://github.com/webpack/webpack/pull/5679
fn scan_context_module_concat_call(expr: &CallExpr) -> Option<(String, String)> {
  if !is_concat_call(expr) {
    return None;
  }
  let prefix_raw = if let Some(prefix) = find_concat_expr_prefix_string(expr) {
    prefix
  } else {
    "".to_string()
  };
  let postfix_raw = if let Some(postfix) = find_concat_expr_postfix_string(expr) {
    postfix
  } else {
    "".to_string()
  };

  if prefix_raw.is_empty() && postfix_raw.is_empty() {
    return None;
  }

  let (context, prefix) = split_context_from_prefix(&prefix_raw);
  let reg = format!("^{prefix}.*{postfix_raw}$");

  Some((context.to_string(), reg))
}

fn is_concat_call(expr: &CallExpr) -> bool {
  match &expr.callee {
    Callee::Expr(box Expr::Member(member_expr)) => {
      if let MemberProp::Ident(ident) = &member_expr.prop {
        if ident.sym != *"concat" {
          return false;
        }
      } else {
        return false;
      }

      if let box Expr::Call(call) = &member_expr.obj {
        return is_concat_call(call);
      }
      true
    }
    _ => false,
  }
}

fn find_concat_expr_prefix_string(expr: &CallExpr) -> Option<String> {
  match &expr.callee {
    Callee::Expr(box Expr::Member(member_expr)) => {
      if let box Expr::Lit(Lit::Str(str)) = &member_expr.obj {
        return Some(str.value.to_string());
      }
      if let box Expr::Lit(Lit::Num(num)) = &member_expr.obj {
        return Some(num.value.to_string());
      }
      if let box Expr::Call(call) = &member_expr.obj {
        return find_concat_expr_prefix_string(call);
      }
      None
    }
    _ => None,
  }
}

fn find_concat_expr_postfix_string(expr: &CallExpr) -> Option<String> {
  expr.args.last().and_then(|arg| {
    if let box Expr::Lit(Lit::Str(str)) = &arg.expr {
      return Some(str.value.to_string());
    }
    if let box Expr::Lit(Lit::Num(num)) = &arg.expr {
      return Some(num.value.to_string());
    }
    None
  })
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
  let mut parts = value.split('.');
  let first = parts.next().expect("should have a last str");
  for part in parts.rev() {
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
  matches!(&expr, Expr::Ident(ident) if ident.sym.eq(first))
}

#[inline]
fn is_hmr_api_call(node: &CallExpr, value: &str) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| match_member_expr(expr, value))
    .unwrap_or_default()
}

fn is_require_context_call(node: &CallExpr) -> bool {
  is_hmr_api_call(node, "require.context")
}

pub fn is_module_hot_accept_call(node: &CallExpr) -> bool {
  is_hmr_api_call(node, "module.hot.accept")
}

pub fn is_module_hot_decline_call(node: &CallExpr) -> bool {
  is_hmr_api_call(node, "module.hot.decline")
}

fn match_import_meta_member_expr(mut expr: &Expr, value: &str) -> bool {
  let mut parts = value.split('.');
  // pop import.meta
  parts.next();
  parts.next();
  for part in parts.rev() {
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

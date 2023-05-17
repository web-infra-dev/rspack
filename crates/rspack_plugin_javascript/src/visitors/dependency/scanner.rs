use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  CommonJsRequireContextDependency, CompilerOptions, ConstDependency, ContextMode, ContextOptions,
  Dependency, DependencyCategory, EsmDynamicImportDependency, ImportContextDependency,
  ModuleDependency, RequireContextDependency, ResourceData, RuntimeGlobals,
};
use rspack_regex::RspackRegex;
use swc_core::common::comments::Comments;
use swc_core::common::{pass::AstNodePath, SyntaxContext};
use swc_core::common::{Span, DUMMY_SP};
use swc_core::ecma::ast::{
  AssignExpr, AssignOp, BinExpr, BinaryOp, CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit,
  MemberExpr, MemberProp, MetaPropExpr, MetaPropKind, ModuleDecl, NewExpr, Pat, PatOrExpr, Tpl,
};
use swc_core::ecma::atoms::{js_word, JsWord};
use swc_core::ecma::utils::{member_expr, quote_ident, quote_str};
use swc_core::ecma::visit::{AstParentNodeRef, VisitAstPath, VisitWithPath};
use swc_core::quote;

use super::{as_parent_path, expr_matcher, is_require_context_call};
use crate::dependency::{
  CommonJSRequireDependency, EsmExportDependency, EsmImportDependency, URLDependency,
};
pub const WEBPACK_HASH: &str = "__webpack_hash__";
pub const WEBPACK_PUBLIC_PATH: &str = "__webpack_public_path__";
pub const WEBPACK_MODULES: &str = "__webpack_modules__";
pub const WEBPACK_RESOURCE_QUERY: &str = "__resourceQuery";
pub const WEBPACK_CHUNK_LOAD: &str = "__webpack_chunk_load__";

pub struct DependencyScanner<'a> {
  pub unresolved_ctxt: &'a SyntaxContext,
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
  pub compiler_options: &'a CompilerOptions,
  pub resource_data: &'a ResourceData,
  pub comments: Option<&'a dyn Comments>,
  pub in_try: bool,
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
      self.add_dependency(Box::new(EsmImportDependency::new(
        source,
        Some(import_decl.span.into()),
        as_parent_path(ast_path),
      )));
    }
  }
  fn add_require(&mut self, call_expr: &CallExpr, ast_path: &AstNodePath<AstParentNodeRef<'_>>) {
    if let Callee::Expr(expr) = &call_expr.callee {
      if let Expr::Ident(ident) = &**expr {
        if "require".eq(&ident.sym) && ident.span.ctxt == *self.unresolved_ctxt {
          {
            if call_expr.args.len() != 1 {
              return;
            }
            if let Some(expr) = call_expr.args.get(0) {
              if expr.spread.is_none() {
                // TemplateLiteral String
                if let Expr::Tpl(tpl) = expr.expr.as_ref()  && tpl.exprs.is_empty(){
                  let s = tpl.quasis.first().expect("should have one quasis").raw.as_ref();
                  let request = JsWord::from(s);
                   self.add_dependency(Box::new(CommonJSRequireDependency::new(
                    request,
                    Some(call_expr.span.into()),
                    as_parent_path(ast_path),
                    self.in_try
                  )));
                  return;
                }
                if let Expr::Lit(Lit::Str(s)) = expr.expr.as_ref() {
                  self.add_dependency(Box::new(CommonJSRequireDependency::new(
                    s.value.clone(),
                    Some(call_expr.span.into()),
                    as_parent_path(ast_path),
                    self.in_try,
                  )));
                  return;
                }
                if let Some((context, reg)) = scanner_context_module(expr.expr.as_ref()) {
                  self.add_dependency(Box::new(CommonJsRequireContextDependency::new(
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
                  )));
                }
              }
            }
          }
        }
      }
    }
  }

  fn try_extract_webpack_chunk_name(&self, first_arg_span_of_import_call: &Span) -> Option<String> {
    use swc_core::common::comments::CommentKind;
    static WEBPACK_CHUNK_NAME_CAPTURE_RE: Lazy<regex::Regex> = Lazy::new(|| {
      regex::Regex::new(r#"webpackChunkName\s*:\s*("(?P<_1>(\./)?([\w0-9_\-\[\]\(\)]+/)*?[\w0-9_\-\[\]\(\)]+)"|'(?P<_2>(\./)?([\w0-9_\-\[\]\(\)]+/)*?[\w0-9_\-\[\]\(\)]+)'|`(?P<_3>(\./)?([\w0-9_\-\[\]\(\)]+/)*?[\w0-9_\-\[\]\(\)]+)`)"#)
        .expect("invalid regex")
    });
    self
      .comments
      .with_leading(first_arg_span_of_import_call.lo, |comments| {
        let ret = comments
          .iter()
          .rev()
          .filter(|c| matches!(c.kind, CommentKind::Block))
          .find_map(|comment| {
            WEBPACK_CHUNK_NAME_CAPTURE_RE
              .captures(&comment.text)
              .and_then(|captures| {
                if let Some(cap) = captures.name("_1") {
                  Some(cap)
                } else if let Some(cap) = captures.name("_2") {
                  Some(cap)
                } else {
                  captures.name("_3")
                }
              })
              .map(|mat| mat.as_str().to_string())
          });
        ret
      })
  }

  fn add_dynamic_import(&mut self, node: &CallExpr, ast_path: &AstNodePath<AstParentNodeRef<'_>>) {
    if let Callee::Import(_) = node.callee {
      if let Some(dyn_imported) = node.args.get(0) {
        if dyn_imported.spread.is_none() {
          match dyn_imported.expr.as_ref() {
            Expr::Lit(Lit::Str(imported)) => {
              let chunk_name = self.try_extract_webpack_chunk_name(&imported.span);
              self.add_dependency(Box::new(EsmDynamicImportDependency::new(
                imported.value.clone(),
                Some(node.span.into()),
                as_parent_path(ast_path),
                chunk_name,
              )));
            }
            Expr::Tpl(tpl) if tpl.quasis.len() == 1 => {
              let chunk_name = self.try_extract_webpack_chunk_name(&tpl.span);
              let request = JsWord::from(
                tpl
                  .quasis
                  .first()
                  .expect("should have one quasis")
                  .raw
                  .to_string(),
              );
              self.add_dependency(Box::new(EsmDynamicImportDependency::new(
                request,
                Some(node.span.into()),
                as_parent_path(ast_path),
                chunk_name,
              )));
            }
            _ => {
              if let Some((context, reg)) = scanner_context_module(dyn_imported.expr.as_ref()) {
                self.add_dependency(Box::new(ImportContextDependency::new(
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
                )));
              }
            }
          }
        }
      }
    }
  }

  // new URL("./foo.png", import.meta.url);
  fn add_new_url(&mut self, new_expr: &NewExpr, ast_path: &AstNodePath<AstParentNodeRef<'_>>) {
    if let Expr::Ident(Ident {
      sym: js_word!("URL"),
      ..
    }) = &*new_expr.callee
    {
      if let Some(args) = &new_expr.args {
        if let (Some(first), Some(second)) = (args.first(), args.get(1)) {
          if let (
            ExprOrSpread {
              spread: None,
              expr: box Expr::Lit(Lit::Str(path)),
            },
            // import.meta.url
            ExprOrSpread {
              spread: None,
              expr:
                box Expr::Member(MemberExpr {
                  obj:
                    box Expr::MetaProp(MetaPropExpr {
                      kind: MetaPropKind::ImportMeta,
                      ..
                    }),
                  prop:
                    MemberProp::Ident(Ident {
                      sym: js_word!("url"),
                      ..
                    }),
                  ..
                }),
            },
          ) = (first, second)
          {
            self.add_dependency(Box::new(URLDependency::new(
              path.value.clone(),
              Some(new_expr.span.into()),
              as_parent_path(ast_path),
            )))
          }
        }
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
        if let Some(src) = &node.src {
          // TODO: this should ignore from code generation or use a new dependency instead
          self.add_dependency(Box::new(EsmExportDependency::new(
            src.value.clone(),
            Some(node.span.into()),
            as_parent_path(ast_path),
          )));
        }
      }
      ModuleDecl::ExportAll(node) => {
        // export * from './other'
        // TODO: this should ignore from code generation or use a new dependency instead
        self.add_dependency(Box::new(EsmExportDependency::new(
          node.src.value.clone(),
          Some(node.span.into()),
          as_parent_path(ast_path),
        )));
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
        self.add_dependency(Box::new(RequireContextDependency::new(
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
        )));
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
      // TODO(ahabhgk): should collected by Diagnostics
      eprintln!("{e}");
    }
    node.visit_children_with_path(self, ast_path);
  }

  fn visit_call_expr<'ast: 'r, 'r>(
    &mut self,
    node: &'ast CallExpr,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    self.add_dynamic_import(node, &*ast_path);
    self.add_require(node, &*ast_path);
    self.scan_require_context(node, &*ast_path);
    node.visit_children_with_path(self, ast_path);
  }

  fn visit_new_expr<'ast: 'r, 'r>(
    &mut self,
    node: &'ast NewExpr,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    self.add_new_url(node, &*ast_path);
    node.visit_children_with_path(self, ast_path);
  }
  fn visit_try_stmt<'ast: 'r, 'r>(
    &mut self,
    node: &'ast swc_core::ecma::ast::TryStmt,
    ast_path: &mut swc_core::ecma::visit::AstNodePath<'r>,
  ) {
    self.in_try = true;
    node.visit_children_with_path(self, ast_path);
    self.in_try = false;
  }

  fn visit_expr<'ast: 'r, 'r>(
    &mut self,
    expr: &'ast Expr,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    if let Expr::Assign(AssignExpr {
      op: AssignOp::Assign,
      left: PatOrExpr::Pat(box Pat::Ident(ident)),
      ..
    }) = expr
    {
      // variable can be assigned
      if ident.span.ctxt == *self.unresolved_ctxt && ident.sym.as_ref() == WEBPACK_PUBLIC_PATH {
        let mut new_expr = expr.clone();
        if let Some(e) = new_expr.as_mut_assign() {
          e.left = PatOrExpr::Pat(Box::new(Pat::Ident(
            quote_ident!(RuntimeGlobals::PUBLIC_PATH).into(),
          )))
        };
        self.add_presentational_dependency(Box::new(ConstDependency::new(
          new_expr,
          Some(RuntimeGlobals::PUBLIC_PATH),
          as_parent_path(ast_path),
        )));
      }
    }

    if let Expr::Ident(ident) = expr {
      // match empty context because the ast of react refresh visitor not resolve mark
      if ident.span.ctxt == *self.unresolved_ctxt || ident.span.ctxt == SyntaxContext::empty() {
        match ident.sym.as_ref() as &str {
          WEBPACK_HASH => {
            self.add_presentational_dependency(Box::new(ConstDependency::new(
              quote!(
                "$name()" as Expr,
                name = quote_ident!(RuntimeGlobals::GET_FULL_HASH)
              ),
              Some(RuntimeGlobals::GET_FULL_HASH),
              as_parent_path(ast_path),
            )));
          }
          WEBPACK_PUBLIC_PATH => {
            self.add_presentational_dependency(Box::new(ConstDependency::new(
              Expr::Ident(quote_ident!(RuntimeGlobals::PUBLIC_PATH)),
              Some(RuntimeGlobals::PUBLIC_PATH),
              as_parent_path(ast_path),
            )));
          }
          WEBPACK_MODULES => {
            self.add_presentational_dependency(Box::new(ConstDependency::new(
              Expr::Ident(quote_ident!(RuntimeGlobals::MODULE_FACTORIES)),
              Some(RuntimeGlobals::MODULE_FACTORIES),
              as_parent_path(ast_path),
            )));
          }
          WEBPACK_RESOURCE_QUERY => {
            if let Some(resource_query) = &self.resource_data.resource_query {
              self.add_presentational_dependency(Box::new(ConstDependency::new(
                Expr::Lit(Lit::Str(quote_str!(resource_query.to_owned()))),
                None,
                as_parent_path(ast_path),
              )));
            }
          }
          WEBPACK_CHUNK_LOAD => {
            self.add_presentational_dependency(Box::new(ConstDependency::new(
              Expr::Ident(quote_ident!(RuntimeGlobals::ENSURE_CHUNK)),
              Some(RuntimeGlobals::ENSURE_CHUNK),
              as_parent_path(ast_path),
            )));
          }
          _ => {}
        }
      }
    } else if expr_matcher::is_require_cache(expr) {
      self.add_presentational_dependency(Box::new(ConstDependency::new(
        *member_expr!(DUMMY_SP, __webpack_require__.c),
        Some(RuntimeGlobals::MODULE_CACHE),
        as_parent_path(ast_path),
      )));
    } else if expr_matcher::is_webpack_module_id(expr) {
      self.add_presentational_dependency(Box::new(ConstDependency::new(
        *member_expr!(DUMMY_SP, module.id),
        Some(RuntimeGlobals::MODULE_CACHE),
        as_parent_path(ast_path),
      )));
    }
    expr.visit_children_with_path(self, ast_path);
  }
}

impl<'a> DependencyScanner<'a> {
  pub fn new(
    unresolved_ctxt: &'a SyntaxContext,
    resource_data: &'a ResourceData,
    compiler_options: &'a CompilerOptions,
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
    comments: Option<&'a dyn Comments>,
  ) -> Self {
    Self {
      in_try: false,
      unresolved_ctxt,
      dependencies,
      presentational_dependencies,
      compiler_options,
      resource_data,
      comments,
    }
  }
}

#[inline]
fn split_context_from_prefix(prefix: String) -> (String, String) {
  if let Some(idx) = prefix.rfind('/') {
    (prefix[..idx].to_string(), format!(".{}", &prefix[idx..]))
  } else {
    (".".to_string(), prefix)
  }
}

fn scanner_context_module(expr: &Expr) -> Option<(String, String)> {
  match expr {
    Expr::Tpl(tpl) if !tpl.exprs.is_empty() => Some(scan_context_module_tpl(tpl)),
    Expr::Bin(bin) => scan_context_module_bin(bin),
    Expr::Call(call) => scan_context_module_concat_call(call),
    _ => None,
  }
}

static META_REG: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"[-\[\]\\/{}()*+?.^$|]").expect("Failed to initialize `MATCH_RESOURCE_REGEX`")
});

#[inline]
fn quote_meta(str: String) -> String {
  META_REG.replace_all(&str, "\\$0").to_string()
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
  let (context, prefix) = split_context_from_prefix(prefix_raw);
  let inner_reg = tpl
    .quasis
    .iter()
    .skip(tpl.quasis.len())
    .skip(1)
    .map(|s| s.raw.to_string() + ".*")
    .collect::<Vec<String>>()
    .join("");
  let reg = format!(
    "^{prefix}.*{inner_reg}{postfix_raw}$",
    prefix = quote_meta(prefix),
    postfix_raw = quote_meta(postfix_raw)
  );
  (context, reg)
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

  let (context, prefix) = split_context_from_prefix(prefix_raw);
  let reg = format!(
    "^{prefix}.*{postfix_raw}$",
    prefix = quote_meta(prefix),
    postfix_raw = quote_meta(postfix_raw)
  );

  Some((context, reg))
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

  let (context, prefix) = split_context_from_prefix(prefix_raw);
  let reg = format!(
    "^{prefix}.*{postfix_raw}$",
    prefix = quote_meta(prefix),
    postfix_raw = quote_meta(postfix_raw)
  );

  Some((context, reg))
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

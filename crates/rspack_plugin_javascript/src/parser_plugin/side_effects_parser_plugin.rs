use std::sync::LazyLock;

use rspack_core::{
  DeferredPureCheck, Dependency, DependencyRange, ModuleDependency, SideEffectsBailoutItemWithSpan,
};
use rspack_util::SpanExt;
use rustc_hash::FxHashSet;
use swc_core::{
  atoms::Atom,
  common::{
    BytePos, Mark, Span, Spanned, SyntaxContext,
    comments::{CommentKind, Comments},
  },
  ecma::{
    ast::{
      ArrowExpr, BlockStmt, BlockStmtOrExpr, Class, ClassMember, Decl, Expr, ExprOrSpread,
      Function, ImportSpecifier, ModuleDecl, ModuleItem, Pat, Program, PropName, Stmt, VarDecl,
      VarDeclKind, VarDeclOrExpr,
    },
    utils::{ExprCtx, ExprExt},
    visit::{Visit, VisitWith},
  },
};

use crate::{
  ClassExt, JavascriptParserPlugin,
  dependency::ESMImportSideEffectDependency,
  parser_plugin::esm_import_dependency_parser_plugin::{ESM_SPECIFIER_TAG, ESMSpecifierData},
  visitors::{JavascriptParser, Statement, TagInfoData, VariableDeclaration},
};

static PURE_COMMENTS: LazyLock<regex::Regex> = LazyLock::new(|| {
  regex::Regex::new("(?s)^\\s*(#|@)__PURE__(?:\\s|$)").expect("Should create the regex")
});
pub struct SideEffectsParserPlugin {
  unresolve_ctxt: SyntaxContext,
  analyze_side_effects_free: bool,
}

impl SideEffectsParserPlugin {
  pub fn new(unresolved_mark: Mark, analyze_side_effects_free: bool) -> Self {
    Self {
      unresolve_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
      analyze_side_effects_free,
    }
  }
}

struct PureAnnotation<'a> {
  side_effects_free: FxHashSet<Atom>,
  parser: &'a JavascriptParser<'a>,
}

fn has_no_side_effects_notation(comments: Option<&dyn Comments>, span: Span) -> bool {
  comments.is_some_and(|comments| comments.has_flag(span.lo, "NO_SIDE_EFFECTS"))
}

impl<'a> Visit for PureAnnotation<'a> {
  fn visit_module_decl(&mut self, node: &ModuleDecl) {
    match &node {
      ModuleDecl::ExportDefaultExpr(default_expr) => {
        if let Some(fn_expr) = default_expr.expr.as_fn_expr()
          && (has_no_side_effects_notation(self.parser.comments, default_expr.span())
            || has_no_side_effects_notation(self.parser.comments, fn_expr.span()))
        {
          if let Some(ident) = &fn_expr.ident {
            self.side_effects_free.insert(ident.sym.clone());
          }
          self.side_effects_free.insert(Atom::from("default"));
        }
      }
      ModuleDecl::ExportDefaultDecl(default_decl) => {
        if let Some(fn_expr) = default_decl.decl.as_fn_expr()
          && (has_no_side_effects_notation(self.parser.comments, default_decl.span())
            || has_no_side_effects_notation(self.parser.comments, fn_expr.span()))
        {
          if let Some(ident) = &fn_expr.ident {
            self.side_effects_free.insert(ident.sym.clone());
          }
          self.side_effects_free.insert(Atom::from("default"));
        }
      }
      ModuleDecl::ExportDecl(export_decl) => {
        if let Some(fn_decl) = export_decl.decl.as_fn_decl()
          && (has_no_side_effects_notation(self.parser.comments, export_decl.span())
            || has_no_side_effects_notation(self.parser.comments, fn_decl.span()))
        {
          self.side_effects_free.insert(fn_decl.ident.sym.clone());
        } else if let Some(var_decl) = export_decl.decl.as_var()
          && matches!(var_decl.kind, VarDeclKind::Const)
          && var_decl.decls.len() == 1
        {
          let const_decl = &var_decl.decls[0];
          if let Some(ident) = const_decl.name.as_ident()
            && let Some(Expr::Fn(fn_expr)) = const_decl.init.as_ref().map(|init| init.as_ref())
            && (has_no_side_effects_notation(self.parser.comments, var_decl.span())
              || has_no_side_effects_notation(self.parser.comments, fn_expr.span())
              || has_no_side_effects_notation(self.parser.comments, export_decl.span()))
          {
            self.side_effects_free.insert(ident.sym.clone());
          } else if let Some(ident) = const_decl.name.as_ident()
            && let Some(Expr::Arrow(fn_expr)) = const_decl.init.as_ref().map(|init| init.as_ref())
            && (has_no_side_effects_notation(self.parser.comments, var_decl.span())
              || has_no_side_effects_notation(self.parser.comments, fn_expr.span())
              || has_no_side_effects_notation(self.parser.comments, export_decl.span()))
          {
            self.side_effects_free.insert(ident.sym.clone());
          }
        }
      }
      _ => {}
    }
  }

  fn visit_stmt(&mut self, node: &Stmt) {
    if let Stmt::Decl(decl) = node {
      match decl {
        Decl::Fn(fn_decl) => {
          if has_no_side_effects_notation(self.parser.comments, fn_decl.span()) {
            self.side_effects_free.insert(fn_decl.ident.sym.clone());
          }
        }
        Decl::Var(var_decl) => {
          /*
          example:
          ```
          /*#__NO_SIDE_EFFECTS__*/ const sideEffectFreeVariable = () => {}
          const sideEffectFreeVariable = /*#__NO_SIDE_EFFECTS__*/ () => {}
          ```
           */
          if matches!(var_decl.kind, VarDeclKind::Const) && var_decl.decls.len() == 1 {
            let const_decl = &var_decl.decls[0];

            if let Some(ident) = const_decl.name.as_ident()
              && let Some(Expr::Fn(fn_expr)) = const_decl.init.as_ref().map(|init| init.as_ref())
              && (has_no_side_effects_notation(self.parser.comments, var_decl.span())
                || has_no_side_effects_notation(self.parser.comments, fn_expr.span()))
            {
              self.side_effects_free.insert(ident.sym.clone());
            } else if let Some(ident) = const_decl.name.as_ident()
              && let Some(Expr::Arrow(fn_expr)) = const_decl.init.as_ref().map(|init| init.as_ref())
              && (has_no_side_effects_notation(self.parser.comments, var_decl.span())
                || has_no_side_effects_notation(self.parser.comments, fn_expr.span()))
            {
              self.side_effects_free.insert(ident.sym.clone());
            }
          }
        }
        _ => {}
      }
    }
  }
}

fn insert_module_export_name(
  name: &swc_core::ecma::ast::ModuleExportName,
  refs: &mut FxHashSet<Atom>,
) {
  match name {
    swc_core::ecma::ast::ModuleExportName::Ident(ident) => {
      refs.insert(ident.sym.clone());
    }
    swc_core::ecma::ast::ModuleExportName::Str(str) => {
      if let Some(atom) = str.value.as_atom() {
        refs.insert(atom.clone());
      }
    }
  }
}

fn function_like_var_decl_names(var_decl: &VarDecl, refs: &mut FxHashSet<Atom>) {
  if !matches!(var_decl.kind, VarDeclKind::Const) {
    return;
  }

  for declarator in &var_decl.decls {
    let Some(ident) = declarator.name.as_ident() else {
      continue;
    };

    if matches!(
      declarator.init.as_deref(),
      Some(Expr::Fn(_)) | Some(Expr::Arrow(_))
    ) {
      refs.insert(ident.sym.clone());
    }
  }
}

fn collect_top_level_function_like_local_refs(program: &Program) -> FxHashSet<Atom> {
  let mut refs = FxHashSet::default();

  match program {
    Program::Module(module) => {
      for item in &module.body {
        match item {
          ModuleItem::Stmt(stmt) => {
            if let Stmt::Decl(decl) = stmt {
              match decl {
                Decl::Fn(fn_decl) => {
                  refs.insert(fn_decl.ident.sym.clone());
                }
                Decl::Var(var_decl) => {
                  function_like_var_decl_names(var_decl, &mut refs);
                }
                _ => {}
              }
            }
          }
          ModuleItem::ModuleDecl(decl) => match decl {
            ModuleDecl::ExportDecl(export_decl) => match &export_decl.decl {
              Decl::Fn(fn_decl) => {
                refs.insert(fn_decl.ident.sym.clone());
              }
              Decl::Var(var_decl) => {
                function_like_var_decl_names(var_decl, &mut refs);
              }
              _ => {}
            },
            _ => {}
          },
        }
      }
    }
    Program::Script(script) => {
      for stmt in &script.body {
        if let Stmt::Decl(decl) = stmt {
          match decl {
            Decl::Fn(fn_decl) => {
              refs.insert(fn_decl.ident.sym.clone());
            }
            Decl::Var(var_decl) => {
              function_like_var_decl_names(var_decl, &mut refs);
            }
            _ => {}
          }
        }
      }
    }
  }

  refs
}

fn collect_exported_side_effects_free_refs(program: &Program) -> FxHashSet<Atom> {
  let mut refs = FxHashSet::default();
  let local_function_like_refs = collect_top_level_function_like_local_refs(program);

  let Program::Module(module) = program else {
    return refs;
  };

  for item in &module.body {
    let ModuleItem::ModuleDecl(decl) = item else {
      continue;
    };

    match decl {
      ModuleDecl::ExportDecl(export_decl) => match &export_decl.decl {
        Decl::Fn(fn_decl) => {
          refs.insert(fn_decl.ident.sym.clone());
        }
        Decl::Var(var_decl) => {
          function_like_var_decl_names(var_decl, &mut refs);
        }
        _ => {}
      },
      ModuleDecl::ExportDefaultDecl(default_decl) => match &default_decl.decl {
        swc_core::ecma::ast::DefaultDecl::Fn(fn_expr) => {
          refs.insert(Atom::from("default"));
          if let Some(ident) = &fn_expr.ident {
            refs.insert(ident.sym.clone());
          }
        }
        swc_core::ecma::ast::DefaultDecl::Class(_)
        | swc_core::ecma::ast::DefaultDecl::TsInterfaceDecl(_) => {}
      },
      ModuleDecl::ExportDefaultExpr(default_expr) => {
        if default_expr.expr.is_fn_expr() || default_expr.expr.is_arrow() {
          refs.insert(Atom::from("default"));
        }
      }
      ModuleDecl::ExportNamed(named_export) => {
        if named_export.src.is_none() {
          for specifier in &named_export.specifiers {
            let swc_core::ecma::ast::ExportSpecifier::Named(named) = specifier else {
              continue;
            };

            let orig = match &named.orig {
              swc_core::ecma::ast::ModuleExportName::Ident(ident) => &ident.sym,
              swc_core::ecma::ast::ModuleExportName::Str(_) => continue,
            };

            if !local_function_like_refs.contains(orig) {
              continue;
            }

            if let Some(exported) = &named.exported {
              insert_module_export_name(exported, &mut refs);
            } else {
              refs.insert(orig.clone());
            }
          }
        }
      }
      _ => {}
    }
  }

  refs
}

fn collect_defined_configured_side_effects_free(
  program: &Program,
  configured_side_effects_free: &[String],
) -> FxHashSet<Atom> {
  let exported_refs = collect_exported_side_effects_free_refs(program);

  configured_side_effects_free
    .iter()
    .filter_map(|name| {
      let atom = Atom::from(name.clone());
      exported_refs.contains(&atom).then_some(atom)
    })
    .collect()
}

fn visit_decl_binding_names(decl: &Decl, f: &mut impl FnMut(&Atom)) {
  match decl {
    Decl::Fn(fn_decl) => f(&fn_decl.ident.sym),
    Decl::Class(class_decl) => f(&class_decl.ident.sym),
    Decl::Var(var_decl) => {
      for ident in var_decl
        .decls
        .iter()
        .filter_map(|declarator| declarator.name.as_ident())
      {
        f(&ident.sym);
      }
    }
    _ => {}
  }
}

fn visit_module_decl_binding_names(decl: &ModuleDecl, f: &mut impl FnMut(&Atom)) {
  match decl {
    ModuleDecl::Import(import_decl) => {
      for specifier in &import_decl.specifiers {
        match specifier {
          ImportSpecifier::Named(named) => f(&named.local.sym),
          ImportSpecifier::Default(default) => f(&default.local.sym),
          ImportSpecifier::Namespace(namespace) => f(&namespace.local.sym),
        }
      }
    }
    ModuleDecl::ExportDecl(export_decl) => visit_decl_binding_names(&export_decl.decl, f),
    ModuleDecl::ExportDefaultDecl(default_decl) => match &default_decl.decl {
      swc_core::ecma::ast::DefaultDecl::Fn(fn_expr) => {
        if let Some(ident) = &fn_expr.ident {
          f(&ident.sym);
        }
        f(&Atom::from("default"));
      }
      swc_core::ecma::ast::DefaultDecl::Class(class_expr) => {
        if let Some(ident) = &class_expr.ident {
          f(&ident.sym);
        }
        f(&Atom::from("default"));
      }
      swc_core::ecma::ast::DefaultDecl::TsInterfaceDecl(_) => {}
    },
    ModuleDecl::ExportDefaultExpr(_) => {
      f(&Atom::from("default"));
    }
    ModuleDecl::ExportNamed(named_export) => {
      if named_export.src.is_none() {
        for specifier in &named_export.specifiers {
          if let swc_core::ecma::ast::ExportSpecifier::Named(named) = specifier {
            if let Some(exported) = &named.exported {
              match exported {
                swc_core::ecma::ast::ModuleExportName::Ident(ident) => f(&ident.sym),
                swc_core::ecma::ast::ModuleExportName::Str(str) => {
                  if let Some(atom) = str.value.as_atom() {
                    f(atom);
                  }
                }
              }
            } else {
              match &named.orig {
                swc_core::ecma::ast::ModuleExportName::Ident(ident) => f(&ident.sym),
                swc_core::ecma::ast::ModuleExportName::Str(str) => {
                  if let Some(atom) = str.value.as_atom() {
                    f(atom);
                  }
                }
              }
            }
          }
        }
      }
    }
    _ => {}
  }
}

fn collect_duplicate_top_level_names(program: &Program) -> FxHashSet<Atom> {
  let mut counts = rustc_hash::FxHashMap::<Atom, usize>::default();
  let mut count_name = |name: &Atom| {
    *counts.entry(name.clone()).or_default() += 1;
  };

  match program {
    Program::Module(module) => {
      for item in &module.body {
        match item {
          ModuleItem::Stmt(Stmt::Decl(decl)) => visit_decl_binding_names(decl, &mut count_name),
          ModuleItem::ModuleDecl(decl) => visit_module_decl_binding_names(decl, &mut count_name),
          _ => {}
        }
      }
    }
    Program::Script(script) => {
      for stmt in &script.body {
        if let Stmt::Decl(decl) = stmt {
          visit_decl_binding_names(decl, &mut count_name);
        }
      }
    }
  }

  counts
    .into_iter()
    .filter_map(|(name, count)| (count > 1).then_some(name))
    .collect()
}

fn mark_side_effects_free(parser: &mut JavascriptParser, name: &Atom, export_name: Option<&Atom>) {
  let side_effects_free = parser.build_info.side_effects_free.get_or_insert_default();
  side_effects_free.insert(name.clone());
  if let Some(export_name) = export_name {
    side_effects_free.insert(export_name.clone());
  }
}

fn try_mark_auto_side_effects_free_var_decl(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  var_decl: &VarDecl,
  export_name: Option<&Atom>,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
  duplicate_names: &FxHashSet<Atom>,
) {
  if !matches!(var_decl.kind, VarDeclKind::Const) {
    return;
  }

  for declarator in &var_decl.decls {
    let Some(ident) = declarator.name.as_ident() else {
      continue;
    };

    if parser
      .build_info
      .side_effects_free
      .as_ref()
      .is_some_and(|side_effects_free| side_effects_free.contains(&ident.sym))
    {
      continue;
    }

    if duplicate_names.contains(&ident.sym) {
      continue;
    }

    let is_side_effects_free = match declarator.init.as_deref() {
      Some(Expr::Fn(fn_expr)) => is_side_effects_free_function_body(
        parser,
        analyze_side_effects_free,
        &fn_expr.function,
        unresolved_ctxt,
        comments,
      ),
      Some(Expr::Arrow(arrow_expr)) => is_side_effects_free_arrow_body(
        parser,
        analyze_side_effects_free,
        arrow_expr,
        unresolved_ctxt,
        comments,
      ),
      _ => false,
    };

    if is_side_effects_free {
      mark_side_effects_free(parser, &ident.sym, export_name);
    }
  }
}

fn try_mark_auto_side_effects_free_stmt(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  stmt: &Stmt,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
  duplicate_names: &FxHashSet<Atom>,
) {
  match stmt {
    Stmt::Decl(Decl::Fn(fn_decl)) => {
      if parser
        .build_info
        .side_effects_free
        .as_ref()
        .is_some_and(|side_effects_free| side_effects_free.contains(&fn_decl.ident.sym))
        || duplicate_names.contains(&fn_decl.ident.sym)
      {
        return;
      }

      if is_side_effects_free_function_body(
        parser,
        analyze_side_effects_free,
        &fn_decl.function,
        unresolved_ctxt,
        comments,
      ) {
        mark_side_effects_free(parser, &fn_decl.ident.sym, None);
      }
    }
    Stmt::Decl(Decl::Var(var_decl)) => try_mark_auto_side_effects_free_var_decl(
      parser,
      analyze_side_effects_free,
      var_decl,
      None,
      unresolved_ctxt,
      comments,
      duplicate_names,
    ),
    _ => {}
  }
}

fn try_mark_auto_side_effects_free_module_decl(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  decl: &ModuleDecl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
  duplicate_names: &FxHashSet<Atom>,
) {
  match decl {
    ModuleDecl::ExportDefaultExpr(default_expr) => {
      let Some(fn_expr) = default_expr.expr.as_fn_expr() else {
        return;
      };
      let Some(ident) = &fn_expr.ident else {
        return;
      };
      let export_name = Atom::from("default");
      if parser
        .build_info
        .side_effects_free
        .as_ref()
        .is_some_and(|side_effects_free| side_effects_free.contains(&ident.sym))
        || duplicate_names.contains(&ident.sym)
      {
        return;
      }
      if is_side_effects_free_function_body(
        parser,
        analyze_side_effects_free,
        &fn_expr.function,
        unresolved_ctxt,
        comments,
      ) {
        mark_side_effects_free(parser, &ident.sym, Some(&export_name));
      }
    }
    ModuleDecl::ExportDefaultDecl(default_decl) => {
      let Some(fn_expr) = default_decl.decl.as_fn_expr() else {
        return;
      };
      let Some(ident) = &fn_expr.ident else {
        return;
      };
      let export_name = Atom::from("default");
      if parser
        .build_info
        .side_effects_free
        .as_ref()
        .is_some_and(|side_effects_free| side_effects_free.contains(&ident.sym))
        || duplicate_names.contains(&ident.sym)
      {
        return;
      }
      if is_side_effects_free_function_body(
        parser,
        analyze_side_effects_free,
        &fn_expr.function,
        unresolved_ctxt,
        comments,
      ) {
        mark_side_effects_free(parser, &ident.sym, Some(&export_name));
      }
    }
    ModuleDecl::ExportDecl(export_decl) => match &export_decl.decl {
      Decl::Fn(fn_decl) => {
        if parser
          .build_info
          .side_effects_free
          .as_ref()
          .is_some_and(|side_effects_free| side_effects_free.contains(&fn_decl.ident.sym))
          || duplicate_names.contains(&fn_decl.ident.sym)
        {
          return;
        }

        if is_side_effects_free_function_body(
          parser,
          analyze_side_effects_free,
          &fn_decl.function,
          unresolved_ctxt,
          comments,
        ) {
          mark_side_effects_free(parser, &fn_decl.ident.sym, None);
        }
      }
      Decl::Var(var_decl) => try_mark_auto_side_effects_free_var_decl(
        parser,
        analyze_side_effects_free,
        var_decl,
        None,
        unresolved_ctxt,
        comments,
        duplicate_names,
      ),
      _ => {}
    },
    _ => {}
  }
}

fn mark_auto_side_effects_free_program(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  program: &Program,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
  duplicate_names: &FxHashSet<Atom>,
) {
  match program {
    Program::Module(module) => {
      for item in &module.body {
        match item {
          ModuleItem::Stmt(stmt) => try_mark_auto_side_effects_free_stmt(
            parser,
            analyze_side_effects_free,
            stmt,
            unresolved_ctxt,
            comments,
            duplicate_names,
          ),
          ModuleItem::ModuleDecl(decl) => try_mark_auto_side_effects_free_module_decl(
            parser,
            analyze_side_effects_free,
            decl,
            unresolved_ctxt,
            comments,
            duplicate_names,
          ),
        }
      }
    }
    Program::Script(script) => {
      for stmt in &script.body {
        try_mark_auto_side_effects_free_stmt(
          parser,
          analyze_side_effects_free,
          stmt,
          unresolved_ctxt,
          comments,
          duplicate_names,
        );
      }
    }
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl JavascriptParserPlugin for SideEffectsParserPlugin {
  fn program(
    &self,
    parser: &mut JavascriptParser,
    ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    parser.build_info.side_effects_free = None;
    parser.build_info.deferred_pure_checks.clear();

    // analyze if any function contains #__NO_SIDE_EFFECTS__ annotation
    // so that pure functions in current module can be marked as pure
    if self.analyze_side_effects_free {
      // Build the explicit/implicit sideEffectsFree set in three steps:
      // 1. collect NO_SIDE_EFFECTS annotations already present in the module,
      // 2. keep only configured names that actually exist at top level,
      // 3. run a fixed-point top-level auto analysis so local pure helpers can
      //    unlock later candidates regardless of declaration order.
      // use a raw swc visitor so that we can find all pure functions before the parser visit the ast
      let mut pure_annotation = PureAnnotation {
        side_effects_free: FxHashSet::default(),
        parser,
      };
      ast.visit_with(&mut pure_annotation);
      let detected_side_effects_free = pure_annotation.side_effects_free;
      if !detected_side_effects_free.is_empty() {
        let side_effects_free = parser.build_info.side_effects_free.get_or_insert_default();
        side_effects_free.extend(detected_side_effects_free);
      }

      if let Some(flagged_side_effects_free) = &parser.javascript_options.side_effects_free {
        let defined_side_effects_free =
          collect_defined_configured_side_effects_free(ast, flagged_side_effects_free);
        if !defined_side_effects_free.is_empty() {
          let side_effects_free = parser.build_info.side_effects_free.get_or_insert_default();
          side_effects_free.extend(defined_side_effects_free);
        }
      }

      let duplicate_names = collect_duplicate_top_level_names(ast);
      loop {
        let prev_len = parser
          .build_info
          .side_effects_free
          .as_ref()
          .map_or(0, FxHashSet::len);
        mark_auto_side_effects_free_program(
          parser,
          self.analyze_side_effects_free,
          ast,
          self.unresolve_ctxt,
          parser.comments,
          &duplicate_names,
        );
        let next_len = parser
          .build_info
          .side_effects_free
          .as_ref()
          .map_or(0, FxHashSet::len);
        if next_len == prev_len {
          break;
        }
      }
    }

    None
  }

  fn module_declaration(&self, parser: &mut JavascriptParser, decl: &ModuleDecl) -> Option<bool> {
    match decl {
      ModuleDecl::ExportDefaultExpr(expr) => {
        let mut callees = vec![];
        if !is_pure_expression(
          parser,
          self.analyze_side_effects_free,
          &expr.expr,
          self.unresolve_ctxt,
          parser.comments,
          Some(&mut callees),
        ) {
          let range = DependencyRange::from(expr.span);
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("ExportDefaultExpr"),
          ));
        } else {
          // record all potential pure callee
          for (callee, span) in callees {
            if let Some(deferred_check) = try_extract_deferred_check(parser, callee, span) {
              parser
                .build_info
                .deferred_pure_checks
                .insert(deferred_check);
            } else {
              let range = DependencyRange::from(span);
              let loc = parser.to_dependency_location(range);
              parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
                range,
                loc,
                String::from("ExportDefaultExpr"),
              ));
              break;
            }
          }
        }
      }
      ModuleDecl::ExportDecl(decl) => {
        let mut callees = vec![];
        if !is_pure_decl(
          parser,
          self.analyze_side_effects_free,
          &decl.decl,
          self.unresolve_ctxt,
          parser.comments,
          Some(&mut callees),
        ) {
          let range = DependencyRange::from(decl.decl.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Decl"),
          ));
        }
        for (callee, span) in callees {
          if let Some(deferred_check) = try_extract_deferred_check(parser, callee, span) {
            parser
              .build_info
              .deferred_pure_checks
              .insert(deferred_check);
          } else {
            let range = DependencyRange::from(span);
            let loc = parser.to_dependency_location(range);
            parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
              range,
              loc,
              String::from("Decl"),
            ));
            break;
          }
        }
      }
      _ => {}
    };
    None
  }
  fn statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    if !parser.is_top_level_scope() {
      return None;
    }
    self.analyze_stmt_side_effects(&stmt, parser);
    None
  }

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    if self.analyze_side_effects_free {
      let mut not_defined = Vec::new();
      // check if all user flagged side_effects_free are defined
      if let Some(side_effects_free) = &parser.javascript_options.side_effects_free {
        let mut side_effects_free = side_effects_free.iter().collect::<Vec<_>>();
        side_effects_free.sort();
        let defined_side_effects_free = parser.build_info.side_effects_free.as_ref();
        for atom in side_effects_free {
          if !defined_side_effects_free.is_some_and(|configured_side_effects_free| {
            configured_side_effects_free.contains(&Atom::from(atom.clone()))
          }) {
            not_defined.push(Atom::from(atom.clone()));
          }
        }
      }

      if !not_defined.is_empty() {
        if let Some(side_effects_free) = parser.build_info.side_effects_free.as_mut() {
          for atom in &not_defined {
            side_effects_free.remove(atom);
          }
        }

        let resource = parser.resource_data.resource();
        parser.add_warning(
          rspack_error::Diagnostic::warn("PURE_FUNCTION_NOT_FOUND".into(), format!("Following pure functions are not found in {resource}:\n[{}]\nRemove it from `module.rules[*].parser.sideEffectsFree`", not_defined.iter().map(|atom| format!("`{atom}`")).collect::<Vec<_>>().join(", ")))
        );
      }
    }

    None
  }
}

#[inline(never)]
fn is_pure_call_expr(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  expr: &Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
  callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  let Expr::Call(call_expr) = expr else {
    unreachable!();
  };
  let pure_flag = has_pure_comment(comments, expr.span().lo)
    || has_pure_comment(comments, call_expr.callee.span().lo);
  let callee = &call_expr.callee;

  if pure_flag {
    return is_pure_call_args(
      parser,
      analyze_side_effects_free,
      call_expr,
      unresolved_ctxt,
      comments,
      callees,
    );
  } else if analyze_side_effects_free
    && let Some(Expr::Ident(ident)) = callee.as_expr().map(|expr| expr.as_ref())
  {
    match resolve_explicit_side_effects_free_callee(
      parser,
      &ident.sym,
      callee.span(),
      callees.is_none(),
    ) {
      ExplicitSideEffectsFreeCallee::Direct => {
        return is_pure_call_args(
          parser,
          analyze_side_effects_free,
          call_expr,
          unresolved_ctxt,
          comments,
          callees,
        );
      }
      ExplicitSideEffectsFreeCallee::Deferred => {
        let Some(callees) = callees else {
          return false;
        };
        callees.push((ident.sym.clone(), callee.span()));
        return is_pure_call_args(
          parser,
          analyze_side_effects_free,
          call_expr,
          unresolved_ctxt,
          comments,
          Some(callees),
        );
      }
      ExplicitSideEffectsFreeCallee::Invalid => return false,
      ExplicitSideEffectsFreeCallee::NotMarked => {}
    }

    if let Some(callees) = callees {
      callees.push((ident.sym.clone(), callee.span()));
      return is_pure_call_args(
        parser,
        analyze_side_effects_free,
        call_expr,
        unresolved_ctxt,
        comments,
        Some(callees),
      );
    }
  }

  !expr.may_have_side_effects(ExprCtx {
    unresolved_ctxt,
    in_strict: false,
    is_unresolved_ref_safe: false,
    remaining_depth: 4,
  })
}

#[inline(never)]
fn is_pure_call_args(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  call_expr: &swc_core::ecma::ast::CallExpr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  for arg in &call_expr.args {
    if arg.spread.is_some() {
      return false;
    }
    if !is_pure_expression(
      parser,
      analyze_side_effects_free,
      &arg.expr,
      unresolved_ctxt,
      comments,
      callees.as_deref_mut(),
    ) {
      return false;
    }
  }
  true
}

enum ExplicitSideEffectsFreeCallee {
  Direct,
  Deferred,
  Invalid,
  NotMarked,
}

fn resolve_explicit_side_effects_free_callee(
  parser: &mut JavascriptParser,
  ident: &Atom,
  span: Span,
  allow_unresolved_marked: bool,
) -> ExplicitSideEffectsFreeCallee {
  let is_marked = parser
    .build_info
    .side_effects_free
    .as_ref()
    .is_some_and(|side_effects_free| side_effects_free.contains(ident));

  if !is_marked {
    return ExplicitSideEffectsFreeCallee::NotMarked;
  }

  if try_extract_deferred_check(parser, ident.clone(), span).is_some() {
    return ExplicitSideEffectsFreeCallee::Deferred;
  }

  if parser.get_variable_info(ident).is_some() || allow_unresolved_marked {
    return ExplicitSideEffectsFreeCallee::Direct;
  }

  ExplicitSideEffectsFreeCallee::Invalid
}

fn try_extract_deferred_check(
  parser: &mut JavascriptParser,
  ident: Atom,
  span: Span,
) -> Option<DeferredPureCheck> {
  let info = parser.get_variable_info(&ident)?;

  let tag_info_id = info.tag_info?;
  let tag_info = parser.definitions_db.expect_get_tag_info(tag_info_id);

  if tag_info.tag != ESM_SPECIFIER_TAG {
    return None;
  }

  let data = ESMSpecifierData::downcast(tag_info.data.clone()?);

  parser
    .get_dependencies()
    .iter()
    .find(|dep| {
      let Some(dep) = dep.downcast_ref::<ESMImportSideEffectDependency>() else {
        return false;
      };

      let request_eq = dep.request() == &data.source;
      let attributes: Option<&rspack_core::ImportAttributes> = data.attributes.as_ref();
      let attributes_eq = attributes == dep.get_attributes();
      request_eq && attributes_eq
    })
    .map(|dep| DeferredPureCheck {
      atom: data
        .ids
        .first()
        .cloned()
        .unwrap_or_else(|| data.name.clone()),
      dep_id: *dep.id(),
      start: span.real_lo(),
      end: span.real_hi(),
    })
}

fn is_pure_new_expr(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  expr: &Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  let Expr::New(new_expr) = expr else {
    unreachable!();
  };
  let pure_flag = has_pure_comment(comments, expr.span().lo);
  if !pure_flag {
    !expr.may_have_side_effects(ExprCtx {
      unresolved_ctxt,
      in_strict: false,
      is_unresolved_ref_safe: false,
      remaining_depth: 4,
    })
  } else {
    are_pure_args(
      parser,
      analyze_side_effects_free,
      new_expr.args.as_deref().unwrap_or(&[]),
      unresolved_ctxt,
      comments,
    )
  }
}

fn has_pure_comment(comments: Option<&dyn Comments>, pos: BytePos) -> bool {
  comments
    .and_then(|comments| comments.get_leading(pos))
    .is_some_and(|comment_list| {
      comment_list
        .iter()
        .any(|comment| comment.kind == CommentKind::Block && PURE_COMMENTS.is_match(&comment.text))
    })
}

fn are_pure_args<'a>(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  args: &'a [ExprOrSpread],
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  args.iter().all(|arg| {
    if arg.spread.is_some() {
      false
    } else {
      is_pure_expression(
        parser,
        analyze_side_effects_free,
        &arg.expr,
        unresolved_ctxt,
        comments,
        None,
      )
    }
  })
}

impl SideEffectsParserPlugin {
  fn analyze_stmt_side_effects(&self, stmt: &Statement, parser: &mut JavascriptParser) {
    if parser.side_effects_item.is_some() {
      return;
    }
    let mut callees = vec![];
    match stmt {
      Statement::If(if_stmt) => {
        if !is_pure_expression(
          parser,
          self.analyze_side_effects_free,
          &if_stmt.test,
          self.unresolve_ctxt,
          parser.comments,
          Some(&mut callees),
        ) {
          let range = DependencyRange::from(if_stmt.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
        }
      }
      Statement::While(while_stmt) => {
        if !is_pure_expression(
          parser,
          self.analyze_side_effects_free,
          &while_stmt.test,
          self.unresolve_ctxt,
          parser.comments,
          Some(&mut callees),
        ) {
          let range = DependencyRange::from(while_stmt.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
        }
      }
      Statement::DoWhile(do_while_stmt) => {
        if !is_pure_expression(
          parser,
          self.analyze_side_effects_free,
          &do_while_stmt.test,
          self.unresolve_ctxt,
          parser.comments,
          Some(&mut callees),
        ) {
          let range = DependencyRange::from(do_while_stmt.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
        }
      }
      Statement::For(for_stmt) => {
        let pure_init = match for_stmt.init {
          Some(ref init) => match init {
            VarDeclOrExpr::VarDecl(decl) => is_pure_var_decl(
              parser,
              self.analyze_side_effects_free,
              decl,
              self.unresolve_ctxt,
              parser.comments,
              Some(&mut callees),
            ),
            VarDeclOrExpr::Expr(expr) => is_pure_expression(
              parser,
              self.analyze_side_effects_free,
              expr,
              self.unresolve_ctxt,
              parser.comments,
              Some(&mut callees),
            ),
          },
          None => true,
        };

        if !pure_init {
          let range = DependencyRange::from(for_stmt.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
          return;
        }

        let pure_test = match &for_stmt.test {
          Some(test) => is_pure_expression(
            parser,
            self.analyze_side_effects_free,
            test,
            self.unresolve_ctxt,
            parser.comments,
            Some(&mut callees),
          ),
          None => true,
        };

        if !pure_test {
          let range = DependencyRange::from(for_stmt.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
          return;
        }

        let pure_update = match for_stmt.update {
          Some(ref expr) => is_pure_expression(
            parser,
            self.analyze_side_effects_free,
            expr,
            self.unresolve_ctxt,
            parser.comments,
            Some(&mut callees),
          ),
          None => true,
        };

        if !pure_update {
          let range = DependencyRange::from(for_stmt.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
        }
      }
      Statement::Expr(expr_stmt) => {
        if !is_pure_expression(
          parser,
          self.analyze_side_effects_free,
          &expr_stmt.expr,
          self.unresolve_ctxt,
          parser.comments,
          Some(&mut callees),
        ) {
          let range = DependencyRange::from(expr_stmt.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
        }
      }
      Statement::Switch(switch_stmt) => {
        if !is_pure_expression(
          parser,
          self.analyze_side_effects_free,
          &switch_stmt.discriminant,
          self.unresolve_ctxt,
          parser.comments,
          Some(&mut callees),
        ) {
          let range = DependencyRange::from(switch_stmt.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
        }
      }
      Statement::Class(class_stmt) => {
        if !is_pure_class(
          parser,
          self.analyze_side_effects_free,
          class_stmt.class(),
          self.unresolve_ctxt,
          parser.comments,
          Some(&mut callees),
        ) {
          let range = DependencyRange::from(class_stmt.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
        }
      }
      Statement::Var(var_stmt) => match var_stmt {
        VariableDeclaration::VarDecl(var_decl) => {
          if !is_pure_var_decl(
            parser,
            self.analyze_side_effects_free,
            var_decl,
            self.unresolve_ctxt,
            parser.comments,
            Some(&mut callees),
          ) {
            let range = DependencyRange::from(var_stmt.span());
            let loc = parser.to_dependency_location(range);
            parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
              range,
              loc,
              String::from("Statement"),
            ));
          }
        }
        VariableDeclaration::UsingDecl(_) => {
          let range = DependencyRange::from(var_stmt.span());
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
        }
      },
      Statement::Empty(_) => {}
      Statement::Labeled(_) => {}
      Statement::Block(_) => {}
      Statement::Fn(_) => {}
      _ => {
        let range = DependencyRange::from(stmt.span());
        let loc = parser.to_dependency_location(range);
        parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
          range,
          loc,
          String::from("Statement"),
        ))
      }
    };

    if parser.side_effects_item.is_none() {
      for (callee, span) in callees {
        if let Some(deferred_check) = try_extract_deferred_check(parser, callee, span) {
          parser
            .build_info
            .deferred_pure_checks
            .insert(deferred_check);
        } else {
          let range = DependencyRange::from(span);
          let loc = parser.to_dependency_location(range);
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            range,
            loc,
            String::from("Statement"),
          ));
          break;
        }
      }
    }
  }
}

pub fn is_pure_pat<'a>(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  pat: &'a Pat,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  match pat {
    Pat::Ident(_) => true,
    Pat::Array(array_pat) => {
      for pat in array_pat.elems.iter().flatten() {
        if !is_pure_pat(
          parser,
          analyze_side_effects_free,
          pat,
          unresolved_ctxt,
          comments,
          callees.as_deref_mut(),
        ) {
          return false;
        }
      }
      true
    }
    Pat::Rest(_) => true,
    Pat::Invalid(_) | Pat::Assign(_) | Pat::Object(_) => false,
    Pat::Expr(expr) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      expr,
      unresolved_ctxt,
      comments,
      callees,
    ),
  }
}

fn is_side_effects_free_param(pat: &Pat) -> bool {
  matches!(pat, Pat::Ident(_))
}

fn function_body_expr_ctx(unresolved_ctxt: SyntaxContext) -> ExprCtx {
  ExprCtx {
    unresolved_ctxt,
    is_unresolved_ref_safe: true,
    in_strict: false,
    remaining_depth: 4,
  }
}

#[inline(never)]
fn is_side_effects_free_var_decl(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  var_decl: &VarDecl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  for declarator in &var_decl.decls {
    if declarator.name.as_ident().is_none() {
      return false;
    }

    if let Some(init) = declarator.init.as_deref()
      && !is_pure_expression(
        parser,
        analyze_side_effects_free,
        init,
        unresolved_ctxt,
        comments,
        None,
      )
    {
      return false;
    }
  }

  true
}

fn stmt_may_have_side_effects(stmt: &Stmt, unresolved_ctxt: SyntaxContext) -> bool {
  let expr_ctx = function_body_expr_ctx(unresolved_ctxt);

  match stmt {
    Stmt::Empty(_) => false,
    Stmt::Expr(expr_stmt) => expr_stmt.expr.may_have_side_effects(expr_ctx),
    Stmt::Return(return_stmt) => return_stmt
      .arg
      .as_deref()
      .is_some_and(|arg| arg.may_have_side_effects(expr_ctx)),
    Stmt::Decl(Decl::Var(var_decl)) => var_decl.decls.iter().any(|declarator| {
      declarator.name.as_ident().is_none()
        || declarator
          .init
          .as_deref()
          .is_some_and(|init| init.may_have_side_effects(expr_ctx))
    }),
    _ => true,
  }
}

fn is_side_effects_free_stmt(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  stmt: &Stmt,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  if !stmt_may_have_side_effects(stmt, unresolved_ctxt) {
    return true;
  }

  match stmt {
    Stmt::Expr(expr_stmt) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      &expr_stmt.expr,
      unresolved_ctxt,
      comments,
      None,
    ),
    Stmt::Return(return_stmt) => return_stmt.arg.as_deref().is_none_or(|arg| {
      is_pure_expression(
        parser,
        analyze_side_effects_free,
        arg,
        unresolved_ctxt,
        comments,
        None,
      )
    }),
    Stmt::Decl(Decl::Var(var_decl)) => is_side_effects_free_var_decl(
      parser,
      analyze_side_effects_free,
      var_decl,
      unresolved_ctxt,
      comments,
    ),
    _ => false,
  }
}

#[inline(never)]
fn is_side_effects_free_block_stmt(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  block_stmt: &BlockStmt,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  for stmt in &block_stmt.stmts {
    if !is_side_effects_free_stmt(
      parser,
      analyze_side_effects_free,
      stmt,
      unresolved_ctxt,
      comments,
    ) {
      return false;
    }
  }

  true
}

#[inline(never)]
fn is_side_effects_free_function_body(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  function: &Function,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  if !function
    .params
    .iter()
    .all(|param| is_side_effects_free_param(&param.pat))
  {
    return false;
  }

  function.body.as_ref().is_none_or(|body| {
    is_side_effects_free_block_stmt(
      parser,
      analyze_side_effects_free,
      body,
      unresolved_ctxt,
      comments,
    )
  })
}

#[inline(never)]
fn is_side_effects_free_arrow_body(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  arrow_expr: &ArrowExpr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  if !arrow_expr.params.iter().all(is_side_effects_free_param) {
    return false;
  }

  match &*arrow_expr.body {
    BlockStmtOrExpr::BlockStmt(block_stmt) => is_side_effects_free_block_stmt(
      parser,
      analyze_side_effects_free,
      block_stmt,
      unresolved_ctxt,
      comments,
    ),
    BlockStmtOrExpr::Expr(expr) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      expr,
      unresolved_ctxt,
      comments,
      None,
    ),
  }
}

pub fn is_pure_function<'a>(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  function: &'a Function,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  for param in &function.params {
    if !is_pure_pat(
      parser,
      analyze_side_effects_free,
      &param.pat,
      unresolved_ctxt,
      comments,
      callees.as_deref_mut(),
    ) {
      return false;
    }
  }
  true
}

#[inline(never)]
pub fn is_pure_expression<'a>(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  expr: &'a Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
  callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  pub fn _is_pure_expression<'a>(
    parser: &mut JavascriptParser,
    analyze_side_effects_free: bool,
    expr: &'a Expr,
    unresolved_ctxt: SyntaxContext,
    comments: Option<&'a dyn Comments>,
    mut callees: Option<&mut Vec<(Atom, Span)>>,
  ) -> bool {
    if let Some(res) = parser.plugin_drive.clone().is_pure(parser, expr) {
      return res;
    }

    match expr {
      Expr::Call(_) => is_pure_call_expr(
        parser,
        analyze_side_effects_free,
        expr,
        unresolved_ctxt,
        comments,
        callees,
      ),
      Expr::New(_) => is_pure_new_expr(
        parser,
        analyze_side_effects_free,
        expr,
        unresolved_ctxt,
        comments,
      ),
      Expr::Paren(_) => unreachable!(),
      Expr::Seq(seq_expr) => {
        for expr in &seq_expr.exprs {
          if !is_pure_expression(
            parser,
            analyze_side_effects_free,
            expr,
            unresolved_ctxt,
            comments,
            callees.as_deref_mut(),
          ) {
            return false;
          }
        }
        true
      }
      _ => !expr.may_have_side_effects(ExprCtx {
        unresolved_ctxt,
        is_unresolved_ref_safe: true,
        in_strict: false,
        remaining_depth: 4,
      }),
    }
  }
  _is_pure_expression(
    parser,
    analyze_side_effects_free,
    expr,
    unresolved_ctxt,
    comments,
    callees,
  )
}

#[inline(never)]
pub fn is_pure_class_member<'a>(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  member: &'a ClassMember,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  let is_key_pure = match member.class_key() {
    Some(PropName::Ident(_ident)) => true,
    Some(PropName::Str(_)) => true,
    Some(PropName::Num(_)) => true,
    Some(PropName::Computed(computed)) => is_pure_expression(
      parser,
      analyze_side_effects_free,
      &computed.expr,
      unresolved_ctxt,
      comments,
      callees.as_deref_mut(),
    ),
    Some(PropName::BigInt(_)) => true,
    None => true,
  };
  if !is_key_pure {
    return false;
  }
  let is_static = member.is_static();
  let is_value_pure = match member {
    ClassMember::Constructor(_) => true,
    ClassMember::Method(_) => true,
    ClassMember::PrivateMethod(_) => true,
    ClassMember::ClassProp(prop) => {
      if let Some(ref value) = prop.value {
        is_pure_expression(
          parser,
          analyze_side_effects_free,
          value,
          unresolved_ctxt,
          comments,
          callees.as_deref_mut(),
        )
      } else {
        true
      }
    }
    ClassMember::PrivateProp(prop) => {
      if let Some(ref value) = prop.value {
        is_pure_expression(
          parser,
          analyze_side_effects_free,
          value,
          unresolved_ctxt,
          comments,
          callees,
        )
      } else {
        true
      }
    }
    ClassMember::TsIndexSignature(_) => unreachable!(),
    ClassMember::Empty(_) => true,
    ClassMember::StaticBlock(_) => false,
    ClassMember::AutoAccessor(_) => false,
  };
  if is_static && !is_value_pure {
    return false;
  }
  true
}

#[inline(never)]
pub fn is_pure_decl(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  stmt: &Decl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
  callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  match stmt {
    Decl::Class(class) => is_pure_class(
      parser,
      analyze_side_effects_free,
      &class.class,
      unresolved_ctxt,
      comments,
      callees,
    ),
    Decl::Fn(_) => true,
    Decl::Var(var) => is_pure_var_decl(
      parser,
      analyze_side_effects_free,
      var,
      unresolved_ctxt,
      comments,
      callees,
    ),
    Decl::Using(_) => false,
    Decl::TsInterface(_) => unreachable!(),
    Decl::TsTypeAlias(_) => unreachable!(),

    Decl::TsEnum(_) => unreachable!(),
    Decl::TsModule(_) => unreachable!(),
  }
}

#[inline(never)]
pub fn is_pure_class(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  class: &Class,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  if let Some(ref super_class) = class.super_class
    && !is_pure_expression(
      parser,
      analyze_side_effects_free,
      super_class,
      unresolved_ctxt,
      comments,
      callees.as_deref_mut(),
    )
  {
    return false;
  }
  let is_pure_key = |parser: &mut JavascriptParser,
                     key: &PropName,
                     callees: Option<&mut Vec<(Atom, Span)>>|
   -> bool {
    match key {
      PropName::BigInt(_) | PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) => true,
      PropName::Computed(computed) => is_pure_expression(
        parser,
        analyze_side_effects_free,
        &computed.expr,
        unresolved_ctxt,
        comments,
        callees,
      ),
    }
  };

  for item in &class.body {
    let pure = match item {
      ClassMember::Constructor(_) => class.super_class.is_none(),
      ClassMember::Method(method) => is_pure_key(parser, &method.key, callees.as_deref_mut()),
      ClassMember::PrivateMethod(method) => is_pure_expression(
        parser,
        analyze_side_effects_free,
        &Expr::PrivateName(method.key.clone()),
        unresolved_ctxt,
        comments,
        callees.as_deref_mut(),
      ),
      ClassMember::ClassProp(prop) => {
        is_pure_key(parser, &prop.key, callees.as_deref_mut())
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(
                parser,
                analyze_side_effects_free,
                value,
                unresolved_ctxt,
                comments,
                callees.as_deref_mut(),
              )
            } else {
              true
            })
      }
      ClassMember::PrivateProp(prop) => {
        is_pure_expression(
          parser,
          analyze_side_effects_free,
          &Expr::PrivateName(prop.key.clone()),
          unresolved_ctxt,
          comments,
          callees.as_deref_mut(),
        ) && (!prop.is_static
          || if let Some(ref value) = prop.value {
            is_pure_expression(
              parser,
              analyze_side_effects_free,
              value,
              unresolved_ctxt,
              comments,
              callees.as_deref_mut(),
            )
          } else {
            true
          })
      }
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => true,
      ClassMember::StaticBlock(_) => false, // TODO: support is pure analyze for statements
      ClassMember::AutoAccessor(_) => false,
    };
    if !pure {
      return false;
    }
  }
  true
}

#[inline(never)]
fn is_pure_var_decl<'a>(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  var: &'a VarDecl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  for decl in &var.decls {
    if let Some(ref init) = decl.init
      && !is_pure_expression(
        parser,
        analyze_side_effects_free,
        init,
        unresolved_ctxt,
        comments,
        callees.as_deref_mut(),
      )
    {
      return false;
    }
  }
  true
}

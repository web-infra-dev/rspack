use rspack_core::{
  DeferredPureCheck, Dependency, ModuleDependency, SideEffectsBailoutItemWithSpan,
};
use rspack_util::SpanExt;
use rustc_hash::FxHashSet;
use swc_core::{
  atoms::Atom,
  common::{Mark, Span, Spanned, SyntaxContext, comments::Comments},
  ecma::{
    ast::{
      Class, ClassMember, Decl, Expr, Function, ModuleDecl, Pat, PropName, Stmt, VarDecl,
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
  comments
    .map(|comments| comments.has_flag(span.lo, "NO_SIDE_EFFECTS"))
    .unwrap_or(false)
}

impl<'a> Visit for PureAnnotation<'a> {
  fn visit_module_decl(&mut self, node: &ModuleDecl) {
    match &node {
      ModuleDecl::ExportDefaultExpr(default_expr) => {
        if let Some(fn_expr) = default_expr.expr.as_fn_expr()
          && let Some(ident) = &fn_expr.ident
          && (has_no_side_effects_notation(self.parser.comments, default_expr.span())
            || has_no_side_effects_notation(self.parser.comments, fn_expr.span()))
        {
          self.side_effects_free.insert(ident.sym.clone());
        }
      }
      ModuleDecl::ExportDefaultDecl(default_decl) => {
        if let Some(fn_expr) = default_decl.decl.as_fn_expr()
          && let Some(ident) = &fn_expr.ident
          && (has_no_side_effects_notation(self.parser.comments, default_decl.span())
            || has_no_side_effects_notation(self.parser.comments, fn_expr.span()))
        {
          self.side_effects_free.insert(ident.sym.clone());
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

impl JavascriptParserPlugin for SideEffectsParserPlugin {
  fn program(
    &self,
    parser: &mut JavascriptParser,
    ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    // analyze if any function contains #__NO_SIDE_EFFECTS__ annotation
    // so that pure functions in current module can be marked as pure
    if self.analyze_side_effects_free {
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
        let side_effects_free = parser.build_info.side_effects_free.get_or_insert_default();
        side_effects_free.extend(flagged_side_effects_free.iter().cloned().map(Into::into));
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            expr.span,
            String::from("ExportDefaultExpr"),
          ));
        } else {
          // record all potential pure callee
          for (callee, span) in callees {
            if let Some(deferred) = try_extract_deferred_check(parser, callee, span) {
              parser.build_info.deferred_pure_checks.insert(deferred);
            } else {
              parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
                span,
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            decl.decl.span(),
            String::from("Decl"),
          ));
        }
        for (callee, span) in callees {
          if let Some(deferred) = try_extract_deferred_check(parser, callee, span) {
            parser.build_info.deferred_pure_checks.insert(deferred);
          } else {
            parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
              span,
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
        for atom in side_effects_free {
          if parser
            .definitions_db
            .get(parser.definitions, &Atom::from(atom.clone()))
            .is_none()
          {
            not_defined.push(atom.clone());
          }
        }
      }

      if !not_defined.is_empty() {
        let resource = parser.resource_data.resource();
        parser.add_warning(
          rspack_error::Diagnostic::warn("PURE_FUNCTION_NOT_FOUND".into(), format!("Following pure functions are not found in {resource}:\n[{}]\nRemove it from `module.rules[{{index}}].sideEffectsFree`", not_defined.iter().map(|atom| format!("`{atom}`")).collect::<Vec<_>>().join(", ")))
        );
      }
    }

    None
  }
}

fn is_pure_call_expr(
  parser: &mut JavascriptParser,
  analyze_side_effects_free: bool,
  expr: &Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
  mut callees: Option<&mut Vec<(Atom, Span)>>,
) -> bool {
  let Expr::Call(call_expr) = expr else {
    unreachable!();
  };

  let callee = &call_expr.callee;
  let pure_flag = comments
    .map(|comments| comments.has_flag(callee.span().lo, "PURE"))
    .unwrap_or(false);

  if pure_flag {
    return call_expr.args.iter().all(|arg| {
      if arg.spread.is_some() {
        false
      } else {
        is_pure_expression(
          parser,
          analyze_side_effects_free,
          &arg.expr,
          unresolved_ctxt,
          comments,
          callees.as_deref_mut(),
        )
      }
    });
  } else if analyze_side_effects_free
    && let Some(Expr::Ident(ident)) = callee.as_expr().map(|expr| expr.as_ref())
  {
    if parser
      .build_info
      .side_effects_free
      .as_ref()
      .map(|side_effects_free| side_effects_free.contains(&ident.sym))
      .unwrap_or(false)
    {
      return true;
    }

    if let Some(callees) = callees {
      callees.push((ident.sym.clone(), callee.span()));

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
          // We CAN pass callees here now because we are in a loop (conceptually),
          // but we need to reborrow.
          // Helper:
          Some(callees),
        ) {
          return false;
        }
      }

      return true;
    }
  }

  !expr.may_have_side_effects(ExprCtx {
    unresolved_ctxt,
    in_strict: false,
    is_unresolved_ref_safe: false,
    remaining_depth: 4,
  })
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
      atom: data.name.clone(),
      dep_id: *dep.id(),
      start: span.real_lo(),
      end: span.real_hi(),
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            if_stmt.span(),
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            while_stmt.span(),
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            do_while_stmt.span(),
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            for_stmt.span(),
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            for_stmt.span(),
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            for_stmt.span(),
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            expr_stmt.span(),
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            switch_stmt.span(),
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
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            class_stmt.span(),
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
            parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
              var_stmt.span(),
              String::from("Statement"),
            ));
          }
        }
        VariableDeclaration::UsingDecl(_) => {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            var_stmt.span(),
            String::from("Statement"),
          ));
        }
      },
      Statement::Empty(_) => {}
      Statement::Labeled(_) => {}
      Statement::Block(_) => {}
      Statement::Fn(_) => {}
      _ => {
        parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
          stmt.span(),
          String::from("Statement"),
        ))
      }
    };

    if parser.side_effects_item.is_none() {
      for (callee, span) in callees {
        if let Some(deferred) = try_extract_deferred_check(parser, callee, span) {
          parser.build_info.deferred_pure_checks.insert(deferred);
        } else {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            span,
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
    let drive = parser.plugin_drive.clone();
    if let Some(res) = drive.is_pure(parser, expr) {
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

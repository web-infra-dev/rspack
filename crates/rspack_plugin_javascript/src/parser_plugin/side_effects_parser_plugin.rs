use rspack_core::{DeferredPureCheck, SideEffectsBailoutItemWithSpan};
use rustc_hash::FxHashSet;
use swc_core::{
  atoms::Atom,
  common::{Mark, Span, Spanned, SyntaxContext, comments::Comments},
  ecma::{
    ast::{
      Callee, Class, ClassMember, Decl, Expr, Function, ModuleDecl, Pat, PropName, Stmt, VarDecl,
      VarDeclKind, VarDeclOrExpr,
    },
    utils::{ExprCtx, ExprExt},
    visit::{Visit, VisitWith},
  },
};

use crate::{
  ClassExt, JavascriptParserPlugin,
  parser_plugin::esm_import_dependency_parser_plugin::{ESM_SPECIFIER_TAG, ESMSpecifierData},
  visitors::{JavascriptParser, Statement, TagInfoData, VariableDeclaration},
};

pub struct SideEffectsParserPlugin {
  unresolve_ctxt: SyntaxContext,
  analyze_pure_notation: bool,
}

impl SideEffectsParserPlugin {
  pub fn new(unresolved_mark: Mark, analyze_pure_notation: bool) -> Self {
    Self {
      unresolve_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
      analyze_pure_notation,
    }
  }
}

struct PureAnnotation<'a> {
  pure_functions: FxHashSet<Atom>,
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
          self.pure_functions.insert(ident.sym.clone());
        }
      }
      ModuleDecl::ExportDefaultDecl(default_decl) => {
        if let Some(fn_expr) = default_decl.decl.as_fn_expr()
          && let Some(ident) = &fn_expr.ident
          && (has_no_side_effects_notation(self.parser.comments, default_decl.span())
            || has_no_side_effects_notation(self.parser.comments, fn_expr.span()))
        {
          self.pure_functions.insert(ident.sym.clone());
        }
      }
      ModuleDecl::ExportDecl(export_decl) => {
        if let Some(fn_decl) = export_decl.decl.as_fn_decl()
          && (has_no_side_effects_notation(self.parser.comments, export_decl.span())
            || has_no_side_effects_notation(self.parser.comments, fn_decl.span()))
        {
          self.pure_functions.insert(fn_decl.ident.sym.clone());
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
            self.pure_functions.insert(ident.sym.clone());
          } else if let Some(ident) = const_decl.name.as_ident()
            && let Some(Expr::Arrow(fn_expr)) = const_decl.init.as_ref().map(|init| init.as_ref())
            && (has_no_side_effects_notation(self.parser.comments, var_decl.span())
              || has_no_side_effects_notation(self.parser.comments, fn_expr.span())
              || has_no_side_effects_notation(self.parser.comments, export_decl.span()))
          {
            self.pure_functions.insert(ident.sym.clone());
          }
        }
      }
      _ => {}
    }
  }

  fn visit_stmt(&mut self, node: &Stmt) {
    match node {
      Stmt::Decl(decl) => match decl {
        Decl::Fn(fn_decl) => {
          if has_no_side_effects_notation(self.parser.comments, fn_decl.span()) {
            self.pure_functions.insert(fn_decl.ident.sym.clone());
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
              self.pure_functions.insert(ident.sym.clone());
            } else if let Some(ident) = const_decl.name.as_ident()
              && let Some(Expr::Arrow(fn_expr)) = const_decl.init.as_ref().map(|init| init.as_ref())
              && (has_no_side_effects_notation(self.parser.comments, var_decl.span())
                || has_no_side_effects_notation(self.parser.comments, fn_expr.span()))
            {
              self.pure_functions.insert(ident.sym.clone());
            }
          }
        }
        _ => {}
      },
      _ => {}
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
    if self.analyze_pure_notation {
      // use a raw swc visitor so that we can find all pure functions before the parser visit the ast
      let mut pure_annotation = PureAnnotation {
        pure_functions: FxHashSet::default(),
        parser,
      };
      ast.visit_with(&mut pure_annotation);
      let detected_pure_functions = pure_annotation.pure_functions;
      if detected_pure_functions.len() > 0 {
        let pure_functions = parser.build_info.pure_functions.get_or_insert_default();
        pure_functions.extend(detected_pure_functions);
      }

      if let Some(flagged_pure_functions) = &parser.javascript_options.pure_functions {
        let pure_functions = parser.build_info.pure_functions.get_or_insert_default();
        pure_functions.extend(flagged_pure_functions.iter().cloned().map(Into::into));
      }
    }

    None
  }

  fn module_declaration(&self, parser: &mut JavascriptParser, decl: &ModuleDecl) -> Option<bool> {
    match decl {
      ModuleDecl::ExportDefaultExpr(expr) => {
        if !is_pure_expression(
          parser,
          self.analyze_pure_notation,
          &expr.expr,
          self.unresolve_ctxt,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            expr.span,
            String::from("ExportDefaultExpr"),
          ));
        }
      }
      ModuleDecl::ExportDecl(decl) => {
        if !is_pure_decl(
          parser,
          self.analyze_pure_notation,
          &decl.decl,
          self.unresolve_ctxt,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            decl.decl.span(),
            String::from("Decl"),
          ));
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
    let mut not_defined = Vec::new();
    // check if all user flagged pure_functions are defined
    if let Some(pure_functions) = &parser.javascript_options.pure_functions {
      let mut pure_functions = pure_functions.into_iter().collect::<Vec<_>>();
      pure_functions.sort();
      for atom in pure_functions {
        if parser
          .definitions_db
          .get(parser.definitions, &Atom::from(atom.clone()))
          .is_none()
        {
          not_defined.push(atom.clone());
        }
      }
    }

    if not_defined.len() > 0 {
      let resource = parser.resource_data.resource();
      parser.add_warning(
        rspack_error::Diagnostic::warn("PURE_FUNCTION_NOT_FOUND".into(), format!("Following pure functions are not found in {resource}:\n[{}]\nRemove it from `module.rules[{{index}}].pureFunctions`", not_defined.iter().map(|atom| format!("`{atom}`")).collect::<Vec<_>>().join(", ")))
      );
    }
    None
  }
}

fn is_pure_call_expr(
  parser: &mut JavascriptParser,
  analyze_pure_notation: bool,
  expr: &Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  let Expr::Call(call_expr) = expr else {
    unreachable!();
  };

  let callee = &call_expr.callee;
  let pure_flag = comments
    .map(|comments| comments.has_flag(callee.span().lo, "PURE"))
    .unwrap_or(false);

  if pure_flag {
    call_expr.args.iter().all(|arg| {
      if arg.spread.is_some() {
        false
      } else {
        is_pure_expression(
          parser,
          analyze_pure_notation,
          &arg.expr,
          unresolved_ctxt,
          comments,
        )
      }
    })
  } else {
    if let Some(Expr::Ident(ident)) = callee.as_expr().map(|expr| expr.as_ref())
      && parser
        .build_info
        .pure_functions
        .as_ref()
        .map(|pure_functions| pure_functions.contains(&ident.sym))
        .unwrap_or(false)
    {
      // this is a locally pure function
      return true;
    }

    if let Some(deferred_check) = try_extract_deferred_check(parser, callee) {
      parser.build_info.deferred_pure_checks.push(deferred_check);
      return call_expr.args.iter().all(|arg| {
        if arg.spread.is_some() {
          false
        } else {
          is_pure_expression(
            parser,
            analyze_pure_notation,
            &arg.expr,
            unresolved_ctxt,
            comments,
          )
        }
      });
    }

    !expr.may_have_side_effects(ExprCtx {
      unresolved_ctxt,
      in_strict: false,
      is_unresolved_ref_safe: false,
      remaining_depth: 4,
    })
  }
}

fn try_extract_deferred_check(
  parser: &mut JavascriptParser,
  callee: &Callee,
) -> Option<DeferredPureCheck> {
  let Callee::Expr(expr) = callee else {
    return None;
  };

  let info = match &**expr {
    Expr::Ident(ident) => parser.get_variable_info(&ident.sym)?,
    _ => return None,
  };

  let tag_info_id = info.tag_info?;
  let tag_info = parser.definitions_db.expect_get_tag_info(tag_info_id);

  if tag_info.tag != ESM_SPECIFIER_TAG {
    return None;
  }

  let data = ESMSpecifierData::downcast(tag_info.data.clone()?);

  Some(DeferredPureCheck {
    import_request: data.source.to_string(),
    atom: data.name.clone(),
    start: callee.span().lo.0,
    end: callee.span().hi.0,
  })
}

impl SideEffectsParserPlugin {
  fn analyze_stmt_side_effects(&self, stmt: &Statement, parser: &mut JavascriptParser) {
    if parser.side_effects_item.is_some() {
      return;
    }
    match stmt {
      Statement::If(if_stmt) => {
        if !is_pure_expression(
          parser,
          self.analyze_pure_notation,
          &if_stmt.test,
          self.unresolve_ctxt,
          parser.comments,
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
          self.analyze_pure_notation,
          &while_stmt.test,
          self.unresolve_ctxt,
          parser.comments,
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
          self.analyze_pure_notation,
          &do_while_stmt.test,
          self.unresolve_ctxt,
          parser.comments,
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
              self.analyze_pure_notation,
              decl,
              self.unresolve_ctxt,
              parser.comments,
            ),
            VarDeclOrExpr::Expr(expr) => is_pure_expression(
              parser,
              self.analyze_pure_notation,
              expr,
              self.unresolve_ctxt,
              parser.comments,
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
            self.analyze_pure_notation,
            test,
            self.unresolve_ctxt,
            parser.comments,
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
            self.analyze_pure_notation,
            expr,
            self.unresolve_ctxt,
            parser.comments,
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
          self.analyze_pure_notation,
          &expr_stmt.expr,
          self.unresolve_ctxt,
          parser.comments,
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
          self.analyze_pure_notation,
          &switch_stmt.discriminant,
          self.unresolve_ctxt,
          parser.comments,
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
          self.analyze_pure_notation,
          class_stmt.class(),
          self.unresolve_ctxt,
          parser.comments,
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
            self.analyze_pure_notation,
            var_decl,
            self.unresolve_ctxt,
            parser.comments,
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
  }
}

pub fn is_pure_pat<'a>(
  parser: &mut JavascriptParser,
  analyze_pure_notation: bool,
  pat: &'a Pat,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  match pat {
    Pat::Ident(_) => true,
    Pat::Array(array_pat) => array_pat.elems.iter().all(|ele| {
      if let Some(pat) = ele {
        is_pure_pat(
          parser,
          analyze_pure_notation,
          pat,
          unresolved_ctxt,
          comments,
        )
      } else {
        true
      }
    }),
    Pat::Rest(_) => true,
    Pat::Invalid(_) | Pat::Assign(_) | Pat::Object(_) => false,
    Pat::Expr(expr) => is_pure_expression(
      parser,
      analyze_pure_notation,
      expr,
      unresolved_ctxt,
      comments,
    ),
  }
}

pub fn is_pure_function<'a>(
  parser: &mut JavascriptParser,
  analyze_pure_notation: bool,
  function: &'a Function,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  if !function.params.iter().all(|param| {
    is_pure_pat(
      parser,
      analyze_pure_notation,
      &param.pat,
      unresolved_ctxt,
      comments,
    )
  }) {
    return false;
  }

  true
}

pub fn is_pure_expression<'a>(
  parser: &mut JavascriptParser,
  analyze_pure_notation: bool,
  expr: &'a Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  pub fn _is_pure_expression<'a>(
    parser: &mut JavascriptParser,
    analyze_pure_notation: bool,
    expr: &'a Expr,
    unresolved_ctxt: SyntaxContext,
    comments: Option<&'a dyn Comments>,
  ) -> bool {
    let drive = parser.plugin_drive.clone();
    if let Some(res) = drive.is_pure(parser, expr) {
      return res;
    }

    match expr {
      Expr::Call(_) => is_pure_call_expr(
        parser,
        analyze_pure_notation,
        expr,
        unresolved_ctxt,
        comments,
      ),
      Expr::Paren(_) => unreachable!(),
      Expr::Seq(seq_expr) => seq_expr.exprs.iter().all(|expr| {
        is_pure_expression(
          parser,
          analyze_pure_notation,
          expr,
          unresolved_ctxt,
          comments,
        )
      }),
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
    analyze_pure_notation,
    expr,
    unresolved_ctxt,
    comments,
  )
}

pub fn is_pure_class_member<'a>(
  parser: &mut JavascriptParser,
  analyze_pure_notation: bool,
  member: &'a ClassMember,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  let is_key_pure = match member.class_key() {
    Some(PropName::Ident(_ident)) => true,
    Some(PropName::Str(_)) => true,
    Some(PropName::Num(_)) => true,
    Some(PropName::Computed(computed)) => is_pure_expression(
      parser,
      analyze_pure_notation,
      &computed.expr,
      unresolved_ctxt,
      comments,
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
          analyze_pure_notation,
          value,
          unresolved_ctxt,
          comments,
        )
      } else {
        true
      }
    }
    ClassMember::PrivateProp(prop) => {
      if let Some(ref value) = prop.value {
        is_pure_expression(
          parser,
          analyze_pure_notation,
          value,
          unresolved_ctxt,
          comments,
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
  analyze_pure_notation: bool,
  stmt: &Decl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  match stmt {
    Decl::Class(class) => is_pure_class(
      parser,
      analyze_pure_notation,
      &class.class,
      unresolved_ctxt,
      comments,
    ),
    Decl::Fn(_) => true,
    Decl::Var(var) => is_pure_var_decl(
      parser,
      analyze_pure_notation,
      var,
      unresolved_ctxt,
      comments,
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
  analyze_pure_notation: bool,
  class: &Class,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  if let Some(ref super_class) = class.super_class
    && !is_pure_expression(
      parser,
      analyze_pure_notation,
      super_class,
      unresolved_ctxt,
      comments,
    )
  {
    return false;
  }
  let is_pure_key = |parser: &mut JavascriptParser, key: &PropName| -> bool {
    match key {
      PropName::BigInt(_) | PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) => true,
      PropName::Computed(computed) => is_pure_expression(
        parser,
        analyze_pure_notation,
        &computed.expr,
        unresolved_ctxt,
        comments,
      ),
    }
  };

  class.body.iter().all(|item| -> bool {
    match item {
      ClassMember::Constructor(_) => class.super_class.is_none(),
      ClassMember::Method(method) => is_pure_key(parser, &method.key),
      ClassMember::PrivateMethod(method) => is_pure_expression(
        parser,
        analyze_pure_notation,
        &Expr::PrivateName(method.key.clone()),
        unresolved_ctxt,
        comments,
      ),
      ClassMember::ClassProp(prop) => {
        is_pure_key(parser, &prop.key)
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(
                parser,
                analyze_pure_notation,
                value,
                unresolved_ctxt,
                comments,
              )
            } else {
              true
            })
      }
      ClassMember::PrivateProp(prop) => {
        is_pure_expression(
          parser,
          analyze_pure_notation,
          &Expr::PrivateName(prop.key.clone()),
          unresolved_ctxt,
          comments,
        ) && (!prop.is_static
          || if let Some(ref value) = prop.value {
            is_pure_expression(
              parser,
              analyze_pure_notation,
              value,
              unresolved_ctxt,
              comments,
            )
          } else {
            true
          })
      }
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => true,
      ClassMember::StaticBlock(_) => false, // TODO: support is pure analyze for statements
      ClassMember::AutoAccessor(_) => false,
    }
  })
}

fn is_pure_var_decl<'a>(
  parser: &mut JavascriptParser,
  analyze_pure_notation: bool,
  var: &'a VarDecl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  var.decls.iter().all(|decl| {
    if let Some(ref init) = decl.init {
      is_pure_expression(
        parser,
        analyze_pure_notation,
        init,
        unresolved_ctxt,
        comments,
      )
    } else {
      true
    }
  })
}

use std::collections::HashMap;

use swc_atoms::{js_word, JsWord};
use swc_common::{
  collections::{AHashMap, AHashSet},
  Mark, SyntaxContext,
};
use swc_ecma_ast::*;
use swc_ecma_visit::{
  as_folder, noop_visit_type, visit_obj_and_computed, Fold, Visit, VisitMut, VisitMutWith,
  VisitWith,
};
use tracing::{debug, span, Level};
use ustr::Ustr;

use crate::{tree_shaking::symbol::Symbol, Dependency, ResolveKind};

const LOG: bool = false && cfg!(debug_assertions);

/// See [Ident] for know how does swc manages identifiers.
///
/// # When to run
///
/// The resolver expects 'clean' ast. You can get clean ast by parsing, or by
/// removing all syntax context in ast nodes.
///
/// # What does it do
///
/// Firstly all scopes (fn, block) has it's own SyntaxContext.
/// Resolver visits all identifiers in module, and look for binding identifies
/// in the scope. Those identifiers now have the SyntaxContext of scope (fn,
/// block). While doing so, resolver tries to resolve normal identifiers (no
/// hygiene info) as a reference to identifier of scope. If the resolver find
/// suitable variable, the identifier reference will have same context as the
/// variable.
///
///
/// # Panics
///
/// `top_level_mark` should not be root.
///
/// # Example
///
/// ```js
/// let a = 1;
/// {
///     let a = 2;
///     use(a);
/// }
/// use(a)
/// ```
///
/// resolver does
///
/// 1.  Define `a` with top level context.
///
/// 2.  Found a block, so visit block with a new syntax context.
///
/// 3. Defined `a` with syntax context of the block statement.
////
/// 4. Found usage of `a`, and determines that it's reference to `a` in the
/// block. So the reference to `a` will have same syntax context as `a` in the
/// block.
///
/// 5. Found usage of `a` (last line), and determines that it's a
/// reference to top-level `a`, and change syntax context of `a` on last line to
/// top-level syntax context.
///
///
/// # Parameters
///
/// ## `unresolved_mark`
///
/// [Mark] applied to unresolved references.
///
/// A pass should accept this [Mark] if it's going to generate a refernce to
/// globals like `require`.
///
/// e.g. `common_js` pass generates calls to `require`, and this should not
/// be shadowed by a declaration named `require` in the same file.
/// So it uses this value.
///
/// ## `top_level_mark`
///
/// [Mark] applied to top-level bindings.
///
/// **NOTE**: This is **not** globals. This is for top level items declared by
/// users.
///
/// A pass should accept this [Mark] if it requires user-defined top-level
/// items.
///
/// e.g. `jsx` pass requires to call `React` imported by the user.
///
/// ```js
/// import React from 'react';
/// ```
///
/// In the code above, `React` has this [Mark]. `jsx` passes need to
/// reference this [Mark], so it accpets this.
///
/// This [Mark] should be used for referencing top-level bindings written by
/// user. If you are going to create a binding, use `private_ident`
/// instead.
///
/// In other words, **this [Mark] should not be used for determining if a
/// variable is top-level.** This is simply a configuration of the `resolver`
/// pass.
///
///
/// ## `typescript`
///
/// Enable this only if you are going to strip types or apply type-aware
/// passes like decorators pass.
///
///
/// # FAQ
///
/// ## Does a pair `(JsWord, SyntaxContext)` always uniquely identifiers a
/// variable binding?
///
/// Yes, but multiple variables can have the exactly same name.
///
/// In the code below,
///
/// ```js
/// var a = 1, a = 2;
/// ```
///
/// both of them have the same name, so the `(JsWord, SyntaxContext)` pair will
/// be also identical.
pub fn resolver(unresolved_mark: Mark, top_level_mark: Mark, typescript: bool) {
  //   assert_ne!(
  //     unresolved_mark,
  //     Mark::root(),
  //     "Marker provided to resolver should not be the root mark"
  //   );

  //   as_folder(Resolver {
  //     current: Scope::new(ScopeKind::Fn, top_level_mark, None),
  //     ident_type: IdentType::Ref,
  //     in_type: false,
  //     in_ts_module: false,
  //     decl_kind: DeclKind::Lexical,
  //     strict_mode: false,
  //     config: InnerConfig {
  //       handle_types: typescript,
  //       unresolved_mark,
  //     },
  //   })
}

/// # Phases
///
/// ## Hoisting phase
///
/// ## Resolving phase
pub struct ModuleRefAnalyze<'a> {
  top_level_mark: Mark,
  unresolved_mark: Mark,
  uri: Ustr,
  dep_to_module_uri: &'a HashMap<Dependency, String>,
}

#[derive(Debug, Clone, Copy)]
struct InnerConfig {
  handle_types: bool,
  unresolved_mark: Mark,
}

impl<'a> ModuleRefAnalyze<'a> {
  pub fn new(
    top_level_mark: Mark,
    unresolved_mark: Mark,
    uri: Ustr,
    dep_to_module_uri: &'a HashMap<Dependency, String>,
  ) -> Self {
    Self {
      top_level_mark,
      unresolved_mark,
      uri,
      dep_to_module_uri,
    }
  }

  //   fn mark_for_ref_inner(&self, sym: &JsWord, stop_an_fn_scope: bool) -> Option<Mark> {
  //     // NaN always points the globals
  //     if *sym == js_word!("NaN") {
  //       return Some(self.config.unresolved_mark);
  //     }

  //     if self.config.handle_types && self.in_type {
  //       let mut mark = self.current.mark;
  //       let mut scope = Some(&self.current);

  //       while let Some(cur) = scope {
  //         // if cur.declared_types.contains(sym) ||
  //         // cur.hoisted_symbols.borrow().contains(sym) {
  //         if cur.declared_types.contains(sym) {
  //           if mark == Mark::root() {
  //             break;
  //           }
  //           return Some(mark);
  //         }

  //         if cur.kind == ScopeKind::Fn && stop_an_fn_scope {
  //           return None;
  //         }

  //         if let Some(parent) = &cur.parent {
  //           mark = parent.mark;
  //         }
  //         scope = cur.parent;
  //       }
  //     }

  //     let mut mark = self.current.mark;
  //     let mut scope = Some(&self.current);

  //     while let Some(cur) = scope {
  //       if cur.declared_symbols.contains_key(sym) {
  //         if mark == Mark::root() {
  //           return None;
  //         }
  //         return Some(mark);
  //       }

  //       if cur.kind == ScopeKind::Fn && stop_an_fn_scope {
  //         return None;
  //       }

  //       if let Some(parent) = &cur.parent {
  //         mark = parent.mark;
  //       }
  //       scope = cur.parent;
  //     }

  //     None
  //   }

  //   /// Modifies a binding identifier.
  //   fn modify(&mut self, ident: &mut Ident, kind: DeclKind) {
  //     if cfg!(debug_assertions) && LOG {
  //       debug!(
  //         "Binding (type = {}) {}{:?} {:?}",
  //         self.in_type,
  //         ident.sym,
  //         ident.span.ctxt(),
  //         kind
  //       );
  //     }

  //     if ident.span.ctxt() != SyntaxContext::empty() {
  //       return;
  //     }

  //     if self.in_type {
  //       self.current.declared_types.insert(ident.sym.clone());
  //       let mark = self.current.mark;

  //       ident.span = if mark == Mark::root() {
  //         ident.span
  //       } else {
  //         let span = ident.span.apply_mark(mark);
  //         if cfg!(debug_assertions) && LOG {
  //           debug!("\t-> {:?}", span.ctxt());
  //         }
  //         span
  //       };
  //       return;
  //     }

  //     let mark = self.current.mark;

  //     self
  //       .current
  //       .declared_symbols
  //       .insert(ident.sym.clone(), kind);

  //     ident.span = if mark == Mark::root() {
  //       ident.span
  //     } else {
  //       let span = ident.span.apply_mark(mark);
  //       if cfg!(debug_assertions) && LOG {
  //         debug!("\t-> {:?}", span.ctxt());
  //       }
  //       span
  //     };
  //   }

  //   fn try_resolving_as_type(&mut self, i: &mut Ident) {
  //     if i.span.ctxt.outer() == self.config.unresolved_mark {
  //       i.span.ctxt = SyntaxContext::empty()
  //     }

  //     self.in_type = true;
  //     i.visit_with(self);
  //     self.in_type = false;
  //   }
}

impl<'a> Visit for ModuleRefAnalyze<'a> {
  noop_visit_type!();
  fn visit_ident(&mut self, node: &Ident) {
    let id = node.to_id();
    let symbol = Symbol::from_id_and_uri(id, self.uri);
    dbg!(symbol);
  }

  fn visit_module_item(&mut self, node: &ModuleItem) {
    match node {
      ModuleItem::ModuleDecl(decl) => match decl {
        ModuleDecl::Import(import) => {
          let src: String = import.src.value.to_string();
          let resolved_uri = self
            .dep_to_module_uri
            .get(&Dependency {
              importer: Some(self.uri.to_string()),
              detail: crate::ModuleDependency {
                specifier: src,
                kind: ResolveKind::Import,
                span: None,
              },
            })
            .unwrap();
          dbg!(&resolved_uri);
          import.specifiers.iter().for_each(|specifier| {});
          // import.visit_with(self);
        }
        ModuleDecl::ExportDecl(decl) => {}
        ModuleDecl::ExportNamed(named_export) => {
          let src: Option<String> = named_export.src.as_ref().map(|src| src.value.to_string());
          if let Some(src) = src {
            let resolved_uri = self
              .dep_to_module_uri
              .get(&Dependency {
                importer: Some(self.uri.to_string()),
                detail: crate::ModuleDependency {
                  specifier: src,
                  kind: ResolveKind::Import,
                  span: None,
                },
              })
              .unwrap();
            dbg!(&resolved_uri);
          } else {
          }
        }
        ModuleDecl::ExportDefaultDecl(_) => todo!(),
        ModuleDecl::ExportDefaultExpr(_) => todo!(),
        ModuleDecl::ExportAll(_) => todo!(),
        ModuleDecl::TsImportEquals(_) => todo!(),
        ModuleDecl::TsExportAssignment(_) => todo!(),
        ModuleDecl::TsNamespaceExport(_) => todo!(),
      },
      ModuleItem::Stmt(_) => node.visit_children_with(self),
    }
  }
}

// struct Hoister<'a, 'b> {
//     resolver: &'a mut Resolver<'b>,
//     kind: DeclKind,
//     /// Hoister should not touch let / const in the block.
//     in_block: bool,

//     in_catch_body: bool,

//     excluded_from_catch: FxHashSet<JsWord>,
//     catch_param_decls: FxHashSet<JsWord>,
// }

// impl Hoister<'_, '_> {
//     fn add_pat_id(&mut self, id: &mut Ident) {
//         if self.in_catch_body {
//             // If we have a binding, it's different variable.
//             if self.resolver.mark_for_ref_inner(&id.sym, true).is_some()
//                 && self.catch_param_decls.contains(&id.sym)
//             {
//                 return;
//             }

//             self.excluded_from_catch.insert(id.sym.clone());
//         } else {
//             // Behavior is different
//             if self.catch_param_decls.contains(&id.sym)
//                 && !self.excluded_from_catch.contains(&id.sym)
//             {
//                 return;
//             }
//         }

//         self.resolver.modify(id, self.kind)
//     }
// }

// impl VisitMut for Hoister<'_, '_> {
//     noop_visit_type!();

//     #[inline]
//     fn visit_arrow_expr(&mut self, _: &mut ArrowExpr) {}

//     fn visit_assign_pat_prop(&mut self, node: &mut AssignPatProp) {
//         node.visit_children_with(self);

//         self.add_pat_id(&mut node.key);
//     }

//     fn visit_block_stmt(&mut self, n: &mut BlockStmt) {
//         let old_in_block = self.in_block;
//         self.in_block = true;
//         n.visit_children_with(self);
//         self.in_block = old_in_block;
//     }

//     /// The code below prints "PASS"
//     ///
//     /// ```js
//     ///
//     ///      var a = "PASS";
//     ///      try {
//     ///          throw "FAIL1";
//     ///          } catch (a) {
//     ///          var a = "FAIL2";
//     ///      }
//     ///      console.log(a);
//     /// ```
//     ///
//     /// While the code below does not throw **ReferenceError** for `b`
//     ///
//     /// ```js
//     ///
//     ///      b()
//     ///      try {
//     ///      } catch (b) {
//     ///          var b;
//     ///      }
//     /// ```
//     ///
//     /// while the code below throws **ReferenceError**
//     ///
//     /// ```js
//     ///
//     ///      b()
//     ///      try {
//     ///      } catch (b) {
//     ///      }
//     /// ```
//     #[inline]
//     fn visit_catch_clause(&mut self, c: &mut CatchClause) {
//         let old_exclude = self.excluded_from_catch.clone();
//         self.excluded_from_catch = Default::default();

//         let old_in_catch_body = self.in_catch_body;

//         let params: Vec<Id> = find_pat_ids(&c.param);

//         self.catch_param_decls
//             .extend(params.into_iter().map(|v| v.0));

//         self.in_catch_body = true;
//         c.body.visit_with(self);

//         let orig = self.catch_param_decls.clone();

//         // let mut excluded = find_ids::<_, Id>(&c.body);

//         // excluded.retain(|id| {
//         //     // If we already have a declartion named a, `var a` in the catch body is
//         //     // different var.

//         //     self.resolver.mark_for_ref(&id.0).is_none()
//         // });

//         self.in_catch_body = false;
//         c.param.visit_with(self);

//         self.catch_param_decls = orig;

//         self.in_catch_body = old_in_catch_body;
//         self.excluded_from_catch = old_exclude;
//     }

//     fn visit_class_decl(&mut self, node: &mut ClassDecl) {
//         if self.in_block {
//             return;
//         }
//         self.resolver.modify(&mut node.ident, DeclKind::Lexical);
//     }

//     #[inline]
//     fn visit_constructor(&mut self, _: &mut Constructor) {}

//     #[inline]
//     fn visit_decl(&mut self, decl: &mut Decl) {
//         decl.visit_children_with(self);

//         if self.resolver.config.handle_types {
//             match decl {
//                 Decl::TsInterface(i) => {
//                     let old_in_type = self.resolver.in_type;
//                     self.resolver.in_type = true;
//                     self.resolver.modify(&mut i.id, DeclKind::Type);
//                     self.resolver.in_type = old_in_type;
//                 }

//                 Decl::TsTypeAlias(a) => {
//                     let old_in_type = self.resolver.in_type;
//                     self.resolver.in_type = true;
//                     self.resolver.modify(&mut a.id, DeclKind::Type);
//                     self.resolver.in_type = old_in_type;
//                 }

//                 Decl::TsEnum(e) => {
//                     if !self.in_block {
//                         let old_in_type = self.resolver.in_type;
//                         self.resolver.in_type = false;
//                         self.resolver.modify(&mut e.id, DeclKind::Lexical);
//                         self.resolver.in_type = old_in_type;
//                     }
//                 }

//                 Decl::TsModule(v)
//                     if matches!(
//                         &**v,
//                         TsModuleDecl {
//                             global: false,
//                             id: TsModuleName::Ident(_),
//                             ..
//                         },
//                     ) =>
//                 {
//                     if !self.in_block {
//                         let old_in_type = self.resolver.in_type;
//                         self.resolver.in_type = false;
//                         self.resolver
//                             .modify(v.id.as_mut_ident().unwrap(), DeclKind::Lexical);
//                         self.resolver.in_type = old_in_type;
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }

//     fn visit_export_default_decl(&mut self, node: &mut ExportDefaultDecl) {
//         // Treat default exported functions and classes as declarations
//         // even though they are parsed as expressions.
//         match &mut node.decl {
//             DefaultDecl::Fn(f) => {
//                 if let Some(id) = &mut f.ident {
//                     self.resolver.modify(id, DeclKind::Var);
//                 }

//                 f.visit_with(self)
//             }
//             DefaultDecl::Class(c) => {
//                 if let Some(id) = &mut c.ident {
//                     self.resolver.modify(id, DeclKind::Lexical);
//                 }

//                 c.visit_with(self)
//             }
//             _ => {
//                 node.visit_children_with(self);
//             }
//         }
//     }

//     #[inline]
//     fn visit_expr(&mut self, _: &mut Expr) {}

//     fn visit_fn_decl(&mut self, node: &mut FnDecl) {
//         if self.catch_param_decls.contains(&node.ident.sym) {
//             return;
//         }

//         if self.in_block {
//             // function declaration is block scoped in strict mode
//             if self.resolver.strict_mode {
//                 return;
//             }
//             // If we are in nested block, and variable named `foo` is lexically declared or
//             // a parameter, we should ignore function foo while handling upper scopes.
//             if let Some(DeclKind::Lexical | DeclKind::Param) =
//                 self.resolver.current.is_declared(&node.ident.sym)
//             {
//                 return;
//             }
//         }

//         self.resolver.modify(&mut node.ident, DeclKind::Function);
//     }

//     #[inline]
//     fn visit_function(&mut self, _: &mut Function) {}

//     fn visit_import_default_specifier(&mut self, n: &mut ImportDefaultSpecifier) {
//         n.visit_children_with(self);

//         self.resolver.modify(&mut n.local, DeclKind::Lexical);
//     }

//     fn visit_import_named_specifier(&mut self, n: &mut ImportNamedSpecifier) {
//         n.visit_children_with(self);

//         self.resolver.modify(&mut n.local, DeclKind::Lexical);
//     }

//     fn visit_import_star_as_specifier(&mut self, n: &mut ImportStarAsSpecifier) {
//         n.visit_children_with(self);

//         self.resolver.modify(&mut n.local, DeclKind::Lexical);
//     }

//     #[inline]
//     fn visit_param(&mut self, _: &mut Param) {}

//     fn visit_pat(&mut self, node: &mut Pat) {
//         match node {
//             Pat::Ident(i) => {
//                 self.add_pat_id(&mut i.id);
//             }

//             _ => node.visit_children_with(self),
//         }
//     }

//     #[inline]
//     fn visit_pat_or_expr(&mut self, _: &mut PatOrExpr) {}

//     #[inline]
//     fn visit_setter_prop(&mut self, _: &mut SetterProp) {}

//     fn visit_switch_stmt(&mut self, s: &mut SwitchStmt) {
//         s.discriminant.visit_with(self);

//         let old_in_block = self.in_block;
//         self.in_block = true;
//         s.cases.visit_with(self);
//         self.in_block = old_in_block;
//     }

//     #[inline]
//     fn visit_tagged_tpl(&mut self, _: &mut TaggedTpl) {}

//     #[inline]
//     fn visit_tpl(&mut self, _: &mut Tpl) {}

//     #[inline]
//     fn visit_ts_module_block(&mut self, _: &mut TsModuleBlock) {}

//     fn visit_var_decl(&mut self, node: &mut VarDecl) {
//         if self.in_block {
//             match node.kind {
//                 VarDeclKind::Const | VarDeclKind::Let => return,
//                 _ => {}
//             }
//         }

//         let old_kind = self.kind;
//         self.kind = node.kind.into();

//         node.visit_children_with(self);

//         self.kind = old_kind;
//     }

//     fn visit_var_decl_or_expr(&mut self, n: &mut VarDeclOrExpr) {
//         match n {
//             VarDeclOrExpr::VarDecl(v)
//                 if matches!(
//                     &**v,
//                     VarDecl {
//                         kind: VarDeclKind::Let | VarDeclKind::Const,
//                         ..
//                     }
//                 ) => {}
//             _ => {
//                 n.visit_children_with(self);
//             }
//         }
//     }

//     fn visit_var_decl_or_pat(&mut self, n: &mut VarDeclOrPat) {
//         match n {
//             VarDeclOrPat::VarDecl(v)
//                 if matches!(
//                     &**v,
//                     VarDecl {
//                         kind: VarDeclKind::Let | VarDeclKind::Const,
//                         ..
//                     }
//                 ) => {}
//             // Hoister should not handle lhs of for in statement below
//             //
//             // const b = [];
//             // {
//             //   let a;
//             //   for (a in b) {
//             //     console.log(a);
//             //   }
//             // }
//             VarDeclOrPat::Pat(..) => {}
//             _ => {
//                 n.visit_children_with(self);
//             }
//         }
//     }

//     #[inline]
//     fn visit_var_declarator(&mut self, node: &mut VarDeclarator) {
//         node.name.visit_with(self);
//     }

//     /// should visit var decls first, cause var decl may appear behind the
//     /// usage. this can deal with code below:
//     /// ```js
//     /// try {} catch (Ic) {
//     ///   throw Ic;
//     /// }
//     /// var Ic;
//     /// ```
//     /// the `Ic` defined by catch param and the `Ic` defined by `var Ic` are
//     /// different variables.
//     /// If we deal with the `var Ic` first, we can know
//     /// that there is already an global declaration of Ic when deal with the try
//     /// block.
//     fn visit_module_items(&mut self, items: &mut Vec<ModuleItem>) {
//         let mut other_items = vec![];

//         for item in items {
//             match item {
//                 ModuleItem::Stmt(Stmt::Decl(Decl::Var(v)))
//                 | ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
//                     decl: Decl::Var(v),
//                     ..
//                 })) if matches!(
//                     &**v,
//                     VarDecl {
//                         kind: VarDeclKind::Var,
//                         ..
//                     }
//                 ) =>
//                 {
//                     item.visit_with(self);
//                 }

//                 ModuleItem::Stmt(Stmt::Decl(Decl::Fn(..)))
//                 | ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
//                     decl: Decl::Fn(..),
//                     ..
//                 })) => {
//                     item.visit_with(self);
//                 }
//                 _ => {
//                     other_items.push(item);
//                 }
//             }
//         }

//         for other_item in other_items {
//             other_item.visit_with(self);
//         }
//     }

//     /// see docs for `self.visit_module_items`
//     fn visit_stmts(&mut self, stmts: &mut Vec<Stmt>) {
//         let mut other_stmts = vec![];

//         for item in stmts {
//             match item {
//                 Stmt::Decl(Decl::Var(v))
//                     if matches!(
//                         &**v,
//                         VarDecl {
//                             kind: VarDeclKind::Var,
//                             ..
//                         }
//                     ) =>
//                 {
//                     item.visit_with(self);
//                 }
//                 Stmt::Decl(Decl::Fn(..)) => {
//                     item.visit_with(self);
//                 }
//                 _ => {
//                     other_stmts.push(item);
//                 }
//             }
//         }

//         for other_stmt in other_stmts {
//             other_stmt.visit_with(self);
//         }
//     }
// }

use std::{collections::HashMap, hash::Hash};

use swc_atoms::{js_word, JsWord};
use swc_common::{
  collections::{AHashMap, AHashSet},
  Mark, SyntaxContext,
};
use swc_ecma_ast::*;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};
use ustr::{ustr, Ustr};

use crate::{tree_shaking::symbol::Symbol, Dependency, ResolveKind};

use super::symbol::IndirectTopLevelSymbol;

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
enum SymbolRef {
  Direct(Symbol),
  Indirect(IndirectTopLevelSymbol),
  /// uri
  Star(Ustr),
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
  export_map: HashMap<JsWord, SymbolRef>,
  /// list of uri, each uri represent export all named export from specific uri
  export_all_list: Vec<Ustr>,
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
      export_map: HashMap::new(),
      export_all_list: vec![],
    }
  }
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
          import.specifiers.iter().for_each(|specifier| {});
          // import.visit_with(self);
        }
        ModuleDecl::ExportDecl(decl) => {}
        ModuleDecl::ExportNamed(named_export) => {
          self.analyze_named_export(named_export);
        }
        ModuleDecl::ExportDefaultDecl(_) => todo!(),
        ModuleDecl::ExportDefaultExpr(_) => todo!(),
        ModuleDecl::ExportAll(export_all) => {
          let resolved_uri_key =
            ustr(self.resolve_uri(export_all.src.value.to_string(), ResolveKind::Import));
          self.export_all_list.push(resolved_uri_key);
        }
        ModuleDecl::TsImportEquals(_) => todo!(),
        ModuleDecl::TsExportAssignment(_) => todo!(),
        ModuleDecl::TsNamespaceExport(_) => todo!(),
      },
      ModuleItem::Stmt(_) => node.visit_children_with(self),
    }
  }
}

impl<'a> ModuleRefAnalyze<'a> {
  fn add_export(&mut self, id: JsWord, symbol: SymbolRef) {
    if self.export_map.contains_key(&id) {
      // TODO: should add some Diagnostic
    } else {
      self.export_map.insert(id, symbol);
    }
  }
  fn analyze_named_export(&mut self, named_export: &NamedExport) {
    let src: Option<String> = named_export.src.as_ref().map(|src| src.value.to_string());
    if let Some(src) = src {
      let resolved_uri = self.resolve_uri(src, ResolveKind::Import);
      let resolved_uri_ukey = ustr(&resolved_uri);
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Namespace(namespace) => {
            // TODO: handle `* as xxx`, do we need a extra binding
            self.export_all_list.push(resolved_uri_ukey);
          }
          ExportSpecifier::Default(_) => {
            // Currently swc does not support syntax like `export v from 'xxx';`
            unreachable!("Module has syntax error should not trigger tree_shaking")
          }
          ExportSpecifier::Named(named) => {
            // TODO: what if the named binding is a unresolved_binding?
            // TODO: handle `as xxx`
            let id = match &named.orig {
              ModuleExportName::Ident(ident) => ident.sym.clone(),
              ModuleExportName::Str(_) => todo!(),
            };
            let symbol_ref =
              SymbolRef::Indirect(IndirectTopLevelSymbol::new(resolved_uri_ukey, id.clone()));
            self.add_export(id, symbol_ref);
          }
        });
    } else {
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Namespace(_) => {
            // named_export has namespace specifier but no src will trigger a syntax error and should not reach here. e.g.
            // `export *`;
            unreachable!("")
          }
          ExportSpecifier::Default(_) => {
            // Currently swc does not support syntax like `export v from 'xxx';`
            unreachable!("Module has syntax error should not trigger tree shaking")
          }
          ExportSpecifier::Named(named) => {
            // TODO: what if the named binding is a unresolved_binding?
            // TODO: handle `as xxx`
            let id = match &named.orig {
              ModuleExportName::Ident(ident) => ident.to_id(),
              // export {'a'} is a syntax error;
              // `export {'a'} from 'xxx'` is not.
              // we know here export has no src,  so this branch should not reachable.
              ModuleExportName::Str(_) => unreachable!(),
            };
            let symbol_ref = SymbolRef::Direct(Symbol::from_id_and_uri(id.clone(), self.uri));
            self.add_export(id.0, symbol_ref);
          }
        });
    };
  }

  /// Try to resolve_uri from `src`, `resolve_kind`, and `importer`
  /// For simplicity, this function will assume the importer is always `self.uri`
  /// # Panic
  /// This function will panic if can't find
  fn resolve_uri(&mut self, src: String, resolve_kind: ResolveKind) -> &String {
    let resolved_uri = self
      .dep_to_module_uri
      .get(&Dependency {
        importer: Some(self.uri.to_string()),
        detail: crate::ModuleDependency {
          specifier: src,
          kind: resolve_kind,
          span: None,
        },
      })
      .unwrap();
    resolved_uri
  }

  fn try_resolve_uri(&mut self, src: String, resolve_kind: ResolveKind) -> Option<&String> {
    self.dep_to_module_uri.get(&Dependency {
      importer: Some(self.uri.to_string()),
      detail: crate::ModuleDependency {
        specifier: src,
        kind: resolve_kind,
        span: None,
      },
    })
  }
}

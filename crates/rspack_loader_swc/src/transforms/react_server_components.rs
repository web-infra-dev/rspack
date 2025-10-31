use std::{iter::FromIterator, rc::Rc, sync::Arc};

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{ClientEntryType, RSCMeta, RSCModuleType};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use swc_core::{
  atoms::{Atom, atom},
  common::{DUMMY_SP, FileName, Span, errors::HANDLER, util::take::Take},
  ecma::{
    ast::*,
    utils::{ExprFactory, prepend_stmts, quote_ident, quote_str},
    visit::{
      Visit, VisitMut, VisitMutWith, VisitWith, noop_visit_mut_type, noop_visit_type,
      visit_mut_pass,
    },
  },
};

use super::{cjs_finder::contains_cjs, import_analyzer::ImportMap};

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Config {
  All(bool),
  WithOptions(Options),
}

impl Config {
  pub fn truthy(&self) -> bool {
    match self {
      Config::All(b) => *b,
      Config::WithOptions(_) => true,
    }
  }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
  pub is_react_server_layer: bool,
}

/// A visitor that transforms given module to use module proxy if it's a React
/// server component.
/// **NOTE** Turbopack uses ClientDirectiveTransformer for the
/// same purpose, so does not run this transform.
struct ReactServerComponents {
  is_react_server_layer: bool,
  filepath: String,
  rsc_meta: Option<RSCMeta>,
  directive_import_collection: Option<(bool, bool, RcVec<ModuleImports>, RcVec<Atom>)>,
}

#[derive(Clone, Debug)]
struct ModuleImports {
  source: (Atom, Span),
  specifiers: Vec<(Atom, Span)>,
}

enum RSCErrorKind {
  /// When `use client` and `use server` are in the same file.
  /// It's not possible to have both directives in the same file.
  RedundantDirectives(Span),
  ErrClientDirective(Span),
  ErrReactApi((String, Span)),
}

impl VisitMut for ReactServerComponents {
  noop_visit_mut_type!();

  fn visit_mut_module(&mut self, module: &mut Module) {
    // Run the validator first to assert, collect directives and imports.
    let mut validator =
      ReactServerComponentValidator::new(self.is_react_server_layer, self.filepath.clone());

    module.visit_with(&mut validator);
    self.directive_import_collection = validator.directive_import_collection;

    let is_client_entry = self
      .directive_import_collection
      .as_ref()
      .expect("directive_import_collection must be set")
      .0;

    self.remove_top_level_directive(module);

    let is_cjs = contains_cjs(module);

    if self.is_react_server_layer {
      if is_client_entry {
        self.to_module_ref(module, is_cjs);
        return;
      }
    } else if is_client_entry {
      self.set_rsc_metadata(is_cjs);
    }
    module.visit_mut_children_with(self)
  }
}

impl ReactServerComponents {
  /// removes specific directive from the AST.
  fn remove_top_level_directive(&mut self, module: &mut Module) {
    module.body.retain(|item| {
      if let ModuleItem::Stmt(stmt) = item {
        if let Some(expr_stmt) = stmt.as_expr() {
          if let Expr::Lit(Lit::Str(Str { value, .. })) = &*expr_stmt.expr {
            if &**value == "use client" {
              // Remove the directive.
              return false;
            }
          }
        }
      }
      true
    });
  }

  // Convert the client module to the module reference code and add a special
  // comment to the top of the file.
  fn to_module_ref(&mut self, module: &mut Module, is_cjs: bool) {
    // Clear all the statements and module declarations.
    module.body.clear();

    let proxy_ident = quote_ident!("createProxy");
    let filepath = quote_str!(&*self.filepath);

    prepend_stmts(
      &mut module.body,
      vec![
        ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
          span: DUMMY_SP,
          kind: VarDeclKind::Const,
          decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Object(ObjectPat {
              span: DUMMY_SP,
              props: vec![ObjectPatProp::Assign(AssignPatProp {
                span: DUMMY_SP,
                key: proxy_ident.into(),
                value: None,
              })],
              optional: false,
              type_ann: None,
            }),
            init: Some(Box::new(Expr::Call(CallExpr {
              span: DUMMY_SP,
              callee: quote_ident!("require").as_callee(),
              args: vec![quote_str!("private-next-rsc-mod-ref-proxy").as_arg()],
              ..Default::default()
            }))),
            definite: false,
          }],
          ..Default::default()
        })))),
        ModuleItem::Stmt(Stmt::Expr(ExprStmt {
          span: DUMMY_SP,
          expr: Box::new(Expr::Assign(AssignExpr {
            span: DUMMY_SP,
            left: MemberExpr {
              span: DUMMY_SP,
              obj: Box::new(Expr::Ident(quote_ident!("module").into())),
              prop: MemberProp::Ident(quote_ident!("exports")),
            }
            .into(),
            op: op!("="),
            right: Box::new(Expr::Call(CallExpr {
              span: DUMMY_SP,
              callee: quote_ident!("createProxy").as_callee(),
              args: vec![filepath.as_arg()],
              ..Default::default()
            })),
          })),
        })),
      ]
      .into_iter(),
    );

    self.set_rsc_metadata(is_cjs);
  }

  fn set_rsc_metadata(&mut self, is_cjs: bool) {
    let export_names = &self
      .directive_import_collection
      .as_ref()
      .expect("directive_import_collection must be set")
      .3;

    self.rsc_meta = Some(RSCMeta {
      module_type: RSCModuleType::Client,
      client_refs: export_names.to_vec(),
      client_entry_type: if is_cjs {
        Some(ClientEntryType::Cjs)
      } else {
        Some(ClientEntryType::Auto)
      },
    });
  }
}

/// Consolidated place to parse, generate error messages for the RSC parsing
/// errors.
fn report_error(error_kind: RSCErrorKind) {
  let (msg, spans) = match error_kind {
    RSCErrorKind::RedundantDirectives(span) => (
      "It's not possible to have both `use client` and `use server` directives in the \
             same file."
        .to_string(),
      vec![span],
    ),
    RSCErrorKind::ErrClientDirective(span) => (
      "The \"use client\" directive must be placed before other expressions. Move it to \
             the top of the file to resolve this issue."
        .to_string(),
      vec![span],
    ),
    RSCErrorKind::ErrReactApi((source, span)) => {
      let msg = if source == "Component" {
        "You’re importing a class component. It only works in a Client Component but none of its parents are marked with \"use client\", so they're Server Components by default.\n\n".to_string()
      } else {
        format!(
          "You're importing a component that needs `{source}`. This React Hook only works in a Client Component. To fix, mark the file (or its parent) with the `\"use client\"` directive.\n\n"
        )
      };

      (msg, vec![span])
    }
  };

  HANDLER.with(|handler| handler.struct_span_err(spans, msg.as_str()).emit())
}

/// Collects top level directives and imports
fn collect_top_level_directives_and_imports(
  module: &Module,
) -> (bool, bool, Vec<ModuleImports>, Vec<Atom>) {
  let mut imports: Vec<ModuleImports> = vec![];
  let mut finished_directives = false;
  let mut is_client_entry = false;
  let mut is_action_file = false;

  let mut export_names = vec![];

  let _ = &module.body.iter().for_each(|item| {
    match item {
      ModuleItem::Stmt(stmt) => {
        if !stmt.is_expr() {
          // Not an expression.
          finished_directives = true;
        }

        match stmt.as_expr() {
          Some(expr_stmt) => {
            match &*expr_stmt.expr {
              Expr::Lit(Lit::Str(Str { value, .. })) => {
                if &**value == "use client" {
                  if !finished_directives {
                    is_client_entry = true;

                    if is_action_file {
                      report_error(RSCErrorKind::RedundantDirectives(expr_stmt.span));
                    }
                  } else {
                    report_error(RSCErrorKind::ErrClientDirective(expr_stmt.span));
                  }
                } else if &**value == "use server" && !finished_directives {
                  is_action_file = true;

                  if is_client_entry {
                    report_error(RSCErrorKind::RedundantDirectives(expr_stmt.span));
                  }
                }
              }
              // Match `ParenthesisExpression` which is some formatting tools
              // usually do: ('use client'). In these case we need to throw
              // an exception because they are not valid directives.
              Expr::Paren(ParenExpr { expr, .. }) => {
                finished_directives = true;
                if let Expr::Lit(Lit::Str(Str { value, .. })) = &**expr {
                  if &**value == "use client" {
                    report_error(RSCErrorKind::ErrClientDirective(expr_stmt.span));
                  }
                }
              }
              _ => {
                // Other expression types.
                finished_directives = true;
              }
            }
          }
          None => {
            // Not an expression.
            finished_directives = true;
          }
        }
      }
      ModuleItem::ModuleDecl(ModuleDecl::Import(
        import @ ImportDecl {
          type_only: false, ..
        },
      )) => {
        let source = import.src.value.clone();
        let specifiers = import
          .specifiers
          .iter()
          .filter(|specifier| {
            !matches!(
              specifier,
              ImportSpecifier::Named(ImportNamedSpecifier {
                is_type_only: true,
                ..
              })
            )
          })
          .map(|specifier| match specifier {
            ImportSpecifier::Named(named) => match &named.imported {
              Some(imported) => match &imported {
                ModuleExportName::Ident(i) => (i.to_id().0, i.span),
                ModuleExportName::Str(s) => (s.value.clone(), s.span),
              },
              None => (named.local.to_id().0, named.local.span),
            },
            ImportSpecifier::Default(d) => (atom!(""), d.span),
            ImportSpecifier::Namespace(n) => (atom!("*"), n.span),
          })
          .collect();

        imports.push(ModuleImports {
          source: (source, import.span),
          specifiers,
        });

        finished_directives = true;
      }
      // Collect all export names.
      ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(e)) => {
        for specifier in &e.specifiers {
          export_names.push(match specifier {
            ExportSpecifier::Default(_) => atom!("default"),
            ExportSpecifier::Namespace(_) => atom!("*"),
            ExportSpecifier::Named(named) => match &named.exported {
              Some(exported) => match &exported {
                ModuleExportName::Ident(i) => i.sym.clone(),
                ModuleExportName::Str(s) => s.value.clone(),
              },
              _ => match &named.orig {
                ModuleExportName::Ident(i) => i.sym.clone(),
                ModuleExportName::Str(s) => s.value.clone(),
              },
            },
          })
        }
        finished_directives = true;
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl { decl, .. })) => {
        match decl {
          Decl::Class(ClassDecl { ident, .. }) => {
            export_names.push(ident.sym.clone());
          }
          Decl::Fn(FnDecl { ident, .. }) => {
            export_names.push(ident.sym.clone());
          }
          Decl::Var(var) => {
            for decl in &var.decls {
              if let Pat::Ident(ident) = &decl.name {
                export_names.push(ident.id.sym.clone());
              }
            }
          }
          _ => {}
        }
        finished_directives = true;
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(ExportDefaultDecl {
        decl: _, ..
      })) => {
        export_names.push(atom!("default"));
        finished_directives = true;
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
        expr: _, ..
      })) => {
        export_names.push(atom!("default"));
        finished_directives = true;
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportAll(_)) => {
        export_names.push(atom!("*"));
      }
      _ => {
        finished_directives = true;
      }
    }
  });

  (is_client_entry, is_action_file, imports, export_names)
}

/// A visitor to assert given module file is a valid React server component.
struct ReactServerComponentValidator {
  is_react_server_layer: bool,
  filepath: String,
  invalid_server_lib_apis_mapping: FxHashMap<&'static str, Vec<&'static str>>,
  pub directive_import_collection: Option<(bool, bool, RcVec<ModuleImports>, RcVec<Atom>)>,
  imports: ImportMap,
}

// A type to workaround a clippy warning.
type RcVec<T> = Rc<Vec<T>>;

impl ReactServerComponentValidator {
  pub fn new(is_react_server_layer: bool, filename: String) -> Self {
    Self {
      is_react_server_layer,
      filepath: filename,
      directive_import_collection: None,
      // react -> [apis]
      // react-dom -> [apis]
      invalid_server_lib_apis_mapping: FxHashMap::from_iter([
        (
          "react",
          vec![
            "Component",
            "createContext",
            "createFactory",
            "PureComponent",
            "useDeferredValue",
            "useEffect",
            "useImperativeHandle",
            "useInsertionEffect",
            "useLayoutEffect",
            "useReducer",
            "useRef",
            "useState",
            "useSyncExternalStore",
            "useTransition",
            "useOptimistic",
            "useActionState",
            "experimental_useOptimistic",
          ],
        ),
        (
          "react-dom",
          vec![
            "flushSync",
            "unstable_batchedUpdates",
            "useFormStatus",
            "useFormState",
          ],
        ),
      ]),
      imports: ImportMap::default(),
    }
  }

  fn is_from_node_modules(&self, filepath: &str) -> bool {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"node_modules[\\/]").unwrap());
    RE.is_match(filepath)
  }

  // Asserts the server lib apis
  // e.g.
  // assert_invalid_server_lib_apis("react", import)
  // assert_invalid_server_lib_apis("react-dom", import)
  fn assert_invalid_server_lib_apis(&self, import_source: String, import: &ModuleImports) {
    let invalid_apis = self
      .invalid_server_lib_apis_mapping
      .get(import_source.as_str());
    if let Some(invalid_apis) = invalid_apis {
      for specifier in &import.specifiers {
        if invalid_apis.contains(&specifier.0.as_str()) {
          report_error(RSCErrorKind::ErrReactApi((
            specifier.0.to_string(),
            specifier.1,
          )));
        }
      }
    }
  }

  fn assert_server_graph(&self, imports: &[ModuleImports]) {
    // If the
    if self.is_from_node_modules(&self.filepath) {
      return;
    }
    for import in imports {
      let source = import.source.0.clone();
      let source_str = source.to_string();
      self.assert_invalid_server_lib_apis(source_str, import);
    }
  }
}

impl Visit for ReactServerComponentValidator {
  noop_visit_type!();

  // coerce parsed script to run validation for the context, which is still
  // required even if file is empty
  fn visit_script(&mut self, script: &swc_core::ecma::ast::Script) {
    if script.body.is_empty() {
      self.visit_module(&Module::dummy());
    }
  }

  fn visit_module(&mut self, module: &Module) {
    self.imports = ImportMap::analyze(module);

    let (is_client_entry, is_action_file, imports, export_names) =
      collect_top_level_directives_and_imports(module);
    let imports = Rc::new(imports);
    let export_names = Rc::new(export_names);

    self.directive_import_collection = Some((
      is_client_entry,
      is_action_file,
      imports.clone(),
      export_names,
    ));

    if self.is_react_server_layer {
      if is_client_entry {
        return;
      } else {
        // Only assert server graph if file's bundle target is "server", e.g.
        // * server components pages
        // * pages bundles on SSR layer
        // * middleware
        // * app/pages api routes
        self.assert_server_graph(&imports);
      }
    }

    module.visit_children_with(self);
  }
}

/// Runs react server component transform for the module proxy, as well as
/// running assertion.
pub fn server_components(filename: Arc<FileName>, config: Config) -> impl Pass + VisitMut {
  let is_react_server_layer: bool = match &config {
    Config::WithOptions(x) => x.is_react_server_layer,
    _ => false,
  };
  visit_mut_pass(ReactServerComponents {
    is_react_server_layer,
    rsc_meta: Some(RSCMeta {
      module_type: RSCModuleType::Server,
      client_refs: vec![],
      client_entry_type: None,
    }),
    filepath: match &*filename {
      FileName::Custom(path) => format!("<{path}>"),
      _ => filename.to_string(),
    },
    directive_import_collection: None,
  })
}

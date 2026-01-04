use std::{cell::RefCell, iter::FromIterator, sync::Arc};

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{RscMeta, RscModuleType};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use swc::atoms::Wtf8Atom;
use swc_core::{
  common::{FileName, Span, errors::HANDLER, util::take::Take},
  ecma::{
    ast::*,
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
  All,
  WithOptions(Options),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
  pub is_react_server_layer: bool,
  pub enable_server_entry: bool,
}

struct DirectiveImportCollection {
  pub is_server_entry: bool,
  pub is_client_entry: bool,
  pub imports: Vec<ModuleImports>,
  pub export_names: Vec<Wtf8Atom>,
}

/// A visitor that transforms given module to use module proxy if it's a React
/// server component.
/// **NOTE** Turbopack uses ClientDirectiveTransformer for the
/// same purpose, so does not run this transform.
struct ReactServerComponents<'a> {
  is_react_server_layer: bool,
  enable_server_entry: bool,
  filepath: String,
  rsc_meta: &'a RefCell<Option<RscMeta>>,
  directive_import_collection: Option<DirectiveImportCollection>,
}

#[derive(Clone, Debug)]
struct ModuleImports {
  source: (Wtf8Atom, Span),
  specifiers: Vec<(Wtf8Atom, Span)>,
}

enum RSCErrorKind {
  /// When `use client` and `use server` are in the same file.
  /// It's not possible to have both directives in the same file.
  RedundantDirectives(Span),
  ErrClientDirective(Span),
  ErrReactApi((String, Span)),
}

impl VisitMut for ReactServerComponents<'_> {
  noop_visit_mut_type!();

  fn visit_mut_module(&mut self, module: &mut Module) {
    // Run the validator first to assert, collect directives and imports.
    let mut validator =
      ReactServerComponentValidator::new(self.is_react_server_layer, self.filepath.clone());

    module.visit_with(&mut validator);
    self.directive_import_collection = validator.directive_import_collection;

    #[allow(clippy::unwrap_used)]
    let directive_import_collection = self.directive_import_collection.as_ref().unwrap();

    let is_server_entry = self.enable_server_entry && directive_import_collection.is_server_entry;
    let is_client_entry = directive_import_collection.is_client_entry;

    self.remove_top_level_directive(module);

    let is_cjs = contains_cjs(module);

    if self.is_react_server_layer {
      if is_server_entry {
        self.set_server_entry_metadata(is_cjs);
      } else if is_client_entry {
        self.set_client_metadata(is_cjs);
      }
    }
    module.visit_mut_children_with(self)
  }
}

impl ReactServerComponents<'_> {
  /// removes specific directive from the AST.
  fn remove_top_level_directive(&mut self, module: &mut Module) {
    module.body.retain(|item| {
      if let ModuleItem::Stmt(stmt) = item
        && let Some(expr_stmt) = stmt.as_expr()
        && let Expr::Lit(Lit::Str(Str { value, .. })) = &*expr_stmt.expr
        && &**value == "use client"
      {
        // Remove the directive.
        return false;
      }
      true
    });
  }

  fn set_server_entry_metadata(&mut self, is_cjs: bool) {
    #[allow(clippy::unwrap_used)]
    let export_names = &self
      .directive_import_collection
      .as_ref()
      .unwrap()
      .export_names;

    let mut rsc_meta = self.rsc_meta.borrow_mut();
    match rsc_meta.as_mut() {
      Some(rsc_meta) => {
        rsc_meta.module_type = RscModuleType::ServerEntry;
        rsc_meta.server_refs = export_names.clone();
        rsc_meta.is_cjs = is_cjs;
      }
      None => {
        *rsc_meta = Some(RscMeta {
          module_type: RscModuleType::ServerEntry,
          server_refs: export_names.clone(),
          client_refs: Default::default(),
          is_cjs,
          action_ids: Default::default(),
        });
      }
    }
  }

  fn set_client_metadata(&mut self, is_cjs: bool) {
    #[allow(clippy::unwrap_used)]
    let export_names = &self
      .directive_import_collection
      .as_ref()
      .unwrap()
      .export_names;

    let mut rsc_meta = self.rsc_meta.borrow_mut();
    match rsc_meta.as_mut() {
      Some(rsc_meta) => {
        rsc_meta.module_type = RscModuleType::Client;
        rsc_meta.client_refs = export_names.clone();
        rsc_meta.is_cjs = is_cjs;
      }
      None => {
        *rsc_meta = Some(RscMeta {
          module_type: RscModuleType::Client,
          server_refs: Default::default(),
          client_refs: export_names.clone(),
          is_cjs,
          action_ids: Default::default(),
        });
      }
    }
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
fn collect_top_level_directives_and_imports(module: &Module) -> DirectiveImportCollection {
  let mut imports: Vec<ModuleImports> = vec![];
  let mut finished_directives = false;
  let mut is_server_entry = false;
  let mut is_client_entry = false;
  let mut is_action_file = false;

  let mut export_names: Vec<Wtf8Atom> = vec![];

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
                if &**value == "use server-entry" {
                  is_server_entry = true;
                } else if &**value == "use client" {
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
                if let Expr::Lit(Lit::Str(Str { value, .. })) = &**expr
                  && &**value == "use client"
                {
                  report_error(RSCErrorKind::ErrClientDirective(expr_stmt.span));
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
                ModuleExportName::Ident(i) => (Wtf8Atom::from(i.to_id().0), i.span),
                ModuleExportName::Str(s) => (s.value.clone(), s.span),
              },
              None => (Wtf8Atom::from(named.local.to_id().0), named.local.span),
            },
            ImportSpecifier::Default(d) => (Wtf8Atom::from(""), d.span),
            ImportSpecifier::Namespace(n) => (Wtf8Atom::from("*"), n.span),
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
            ExportSpecifier::Default(_) => Wtf8Atom::from("default"),
            ExportSpecifier::Namespace(_) => Wtf8Atom::from("*"),
            ExportSpecifier::Named(named) => match &named.exported {
              Some(exported) => match &exported {
                ModuleExportName::Ident(i) => Wtf8Atom::from(i.sym.clone()),
                ModuleExportName::Str(s) => s.value.clone(),
              },
              _ => match &named.orig {
                ModuleExportName::Ident(i) => Wtf8Atom::from(i.sym.clone()),
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
            export_names.push(Wtf8Atom::from(ident.sym.clone()));
          }
          Decl::Fn(FnDecl { ident, .. }) => {
            export_names.push(Wtf8Atom::from(ident.sym.clone()));
          }
          Decl::Var(var) => {
            for decl in &var.decls {
              if let Pat::Ident(ident) = &decl.name {
                export_names.push(Wtf8Atom::from(ident.id.sym.clone()));
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
        export_names.push(Wtf8Atom::from("default"));
        finished_directives = true;
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
        expr: _, ..
      })) => {
        export_names.push(Wtf8Atom::from("default"));
        finished_directives = true;
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportAll(_)) => {
        export_names.push(Wtf8Atom::from("*"));
      }
      _ => {
        finished_directives = true;
      }
    }
  });

  DirectiveImportCollection {
    is_server_entry,
    is_client_entry,
    imports,
    export_names,
  }
}

/// A visitor to assert given module file is a valid React server component.
struct ReactServerComponentValidator {
  is_react_server_layer: bool,
  filepath: String,
  invalid_server_lib_apis_mapping: FxHashMap<&'static str, Vec<&'static str>>,
  pub directive_import_collection: Option<DirectiveImportCollection>,
  imports: ImportMap,
}

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
    static RE: Lazy<Regex> = Lazy::new(|| {
      #[allow(clippy::unwrap_used)]
      Regex::new(r"node_modules[\\/]").unwrap()
    });
    RE.is_match(filepath)
  }

  // Asserts the server lib apis
  // e.g.
  // assert_invalid_server_lib_apis("react", import)
  // assert_invalid_server_lib_apis("react-dom", import)
  fn assert_invalid_server_lib_apis(&self, import_source: &str, import: &ModuleImports) {
    let invalid_apis = self.invalid_server_lib_apis_mapping.get(import_source);
    if let Some(invalid_apis) = invalid_apis {
      for specifier in &import.specifiers {
        if let Some(specifier_name) = specifier.0.as_str()
          && invalid_apis.contains(&specifier_name)
        {
          report_error(RSCErrorKind::ErrReactApi((
            specifier_name.to_string(),
            specifier.1,
          )));
        }
      }
    }
  }

  fn assert_server_graph(&self, imports: &[ModuleImports]) {
    if self.is_from_node_modules(&self.filepath) {
      return;
    }
    for import in imports {
      let source = import.source.0.clone();
      if let Some(source_str) = source.as_str() {
        self.assert_invalid_server_lib_apis(source_str, import);
      }
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

    let directive_import_collection = collect_top_level_directives_and_imports(module);

    if self.is_react_server_layer && !directive_import_collection.is_client_entry {
      // Only assert server graph if file's bundle target is "server", e.g.
      // * server components pages
      // * pages bundles on SSR layer
      // * middleware
      // * app/pages api routes
      self.assert_server_graph(&directive_import_collection.imports)
    }
    self.directive_import_collection = Some(directive_import_collection);

    module.visit_children_with(self);
  }
}

/// Runs react server component transform for the module proxy, as well as
/// running assertion.
pub fn server_components(
  filename: Arc<FileName>,
  config: Config,
  rsc_meta: &RefCell<Option<RscMeta>>,
) -> impl Pass + VisitMut {
  let is_react_server_layer: bool = match &config {
    Config::WithOptions(x) => x.is_react_server_layer,
    _ => false,
  };
  let enable_server_entry = match &config {
    Config::WithOptions(x) => x.enable_server_entry,
    _ => false,
  };
  visit_mut_pass(ReactServerComponents {
    is_react_server_layer,
    enable_server_entry,
    rsc_meta,
    filepath: match &*filename {
      FileName::Custom(path) => format!("<{path}>"),
      _ => filename.to_string(),
    },
    directive_import_collection: None,
  })
}

use rspack_util::json_stringify_str;
use swc::atoms::Wtf8Atom;
use swc_core::{
  atoms::Atom,
  common::{DUMMY_SP, Span, SyntaxContext},
  ecma::{ast::*, utils::ExprFactory},
};

const RSC_SERVER_MODULE: &str = "react-server-dom-rspack/server";
const REGISTER_CLIENT_REFERENCE: &str = "registerClientReference";
const DYNAMIC_EXPORT_BINDING_PREFIX: &str = "__rspack_export_";

/// Replaces a `"use client"` module on the RSC server layer with client
/// reference proxy exports.
///
/// For ESM modules with `default` and `Button` exports, this generates code
/// with this shape:
///
/// ```text
/// import { registerClientReference } from "react-server-dom-rspack/server";
///
/// export default registerClientReference(
///   function() { throw new Error(...); },
///   resource,
///   "default"
/// );
/// export const Button = registerClientReference(
///   function() { throw new Error(...); },
///   resource,
///   "Button"
/// );
/// ```
///
/// CJS modules import through `require` and assign the references directly to
/// `module.exports` / `exports[exportName]`:
///
/// ```text
/// const { registerClientReference } = require("react-server-dom-rspack/server");
///
/// module.exports = registerClientReference(
///   function() { throw new Error(...); },
///   resource,
///   "default"
/// );
/// exports["Button"] = registerClientReference(
///   function() { throw new Error(...); },
///   resource,
///   "Button"
/// );
/// ```
///
/// Returns `false` for `export *` client refs, so the caller can keep the
/// original module and report the unsupported whole-module reference later.
pub fn to_client_ref(
  module: &mut swc_core::ecma::ast::Module,
  resource: &str,
  client_refs: &[Wtf8Atom],
  is_cjs: bool,
) -> bool {
  if client_refs
    .iter()
    .any(|client_ref| client_ref.as_str() == Some("*"))
  {
    return false;
  }

  module.body = if is_cjs {
    to_cjs_client_ref(resource, client_refs)
  } else {
    to_esm_client_ref(resource, client_refs)
  };
  true
}

fn to_esm_client_ref(resource: &str, client_refs: &[Wtf8Atom]) -> Vec<ModuleItem> {
  to_client_ref_module(
    resource,
    client_refs,
    import_named(RSC_SERVER_MODULE, &[REGISTER_CLIENT_REFERENCE]),
    false,
  )
}

fn to_cjs_client_ref(resource: &str, client_refs: &[Wtf8Atom]) -> Vec<ModuleItem> {
  to_client_ref_module(
    resource,
    client_refs,
    const_object_decl(
      &[REGISTER_CLIENT_REFERENCE],
      require_call(RSC_SERVER_MODULE),
    ),
    true,
  )
}

fn to_client_ref_module(
  resource: &str,
  client_refs: &[Wtf8Atom],
  register_client_reference_decl: ModuleItem,
  is_cjs: bool,
) -> Vec<ModuleItem> {
  let mut items = Vec::with_capacity(client_refs.len() + 1);
  items.push(register_client_reference_decl);
  items.extend(client_exports(resource, client_refs, is_cjs));
  items
}

fn client_exports(resource: &str, client_refs: &[Wtf8Atom], is_cjs: bool) -> Vec<ModuleItem> {
  let call_error = client_reference_call_error(resource);
  let mut items = Vec::with_capacity(client_refs.len());
  let mut dynamic_export_count = 0;

  for export_name in client_refs
    .iter()
    .filter_map(|client_ref| client_ref.as_str())
  {
    let reference = register_client_reference_expr(resource, export_name, &call_error);
    items.extend(client_export_decls(
      export_name,
      reference,
      client_refs,
      &mut dynamic_export_count,
      is_cjs,
    ));
  }

  items
}

fn client_export_decls(
  export_name: &str,
  reference: Expr,
  client_refs: &[Wtf8Atom],
  dynamic_export_count: &mut usize,
  is_cjs: bool,
) -> Vec<ModuleItem> {
  match (is_cjs, export_name) {
    (false, "default") => vec![ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
      ExportDefaultExpr {
        span: DUMMY_SP,
        expr: Box::new(reference),
      },
    ))],
    (true, "default") => vec![assign_member_stmt("module", "exports", reference)],
    (false, ident) if Ident::verify_symbol(ident).is_ok() => {
      vec![export_const_decl(ident, reference)]
    }
    (false, export_name) => {
      let local_name = next_dynamic_export_binding(client_refs, dynamic_export_count);
      vec![
        const_decl(&local_name, reference),
        export_named_decl(&local_name, export_name),
      ]
    }
    (true, ident) => vec![assign_computed_member_stmt(
      "exports",
      str_expr(ident),
      reference,
    )],
  }
}

fn client_reference_call_error(resource: &str) -> String {
  format!(
    "Attempted to call the default export of {} from \
    the server, but it's on the client. It's not possible to invoke a \
    client function from the server, it can only be rendered as a \
    Component or passed to props of a Client Component.",
    json_stringify_str(resource)
  )
}

fn next_dynamic_export_binding(
  client_refs: &[Wtf8Atom],
  dynamic_export_count: &mut usize,
) -> String {
  loop {
    *dynamic_export_count += 1;
    let name = format!("{DYNAMIC_EXPORT_BINDING_PREFIX}{dynamic_export_count}__");
    if !client_refs
      .iter()
      .any(|client_ref| client_ref.as_str() == Some(name.as_str()))
    {
      return name;
    }
  }
}

fn register_client_reference_expr(resource: &str, export_name: &str, call_error: &str) -> Expr {
  call_expr(
    ident_expr(REGISTER_CLIENT_REFERENCE),
    vec![
      throw_error_function(call_error),
      str_expr(resource),
      str_expr(export_name),
    ],
    DUMMY_SP,
  )
}

fn throw_error_function(call_error: &str) -> Expr {
  Expr::Fn(FnExpr {
    ident: None,
    function: Box::new(Function {
      body: Some(BlockStmt {
        span: DUMMY_SP,
        stmts: vec![Stmt::Throw(ThrowStmt {
          span: DUMMY_SP,
          arg: Box::new(Expr::New(NewExpr {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            callee: Box::new(ident_expr("Error")),
            args: Some(vec![expr_arg(str_expr(call_error))]),
            type_args: None,
          })),
        })],
        ..Default::default()
      }),
      ..Default::default()
    }),
  })
}

fn import_named(source: &str, names: &[&str]) -> ModuleItem {
  ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
    span: DUMMY_SP,
    specifiers: names
      .iter()
      .map(|name| {
        ImportSpecifier::Named(ImportNamedSpecifier {
          span: DUMMY_SP,
          local: ident(name),
          imported: None,
          is_type_only: false,
        })
      })
      .collect(),
    src: Box::new(Str {
      span: DUMMY_SP,
      value: Wtf8Atom::from(source),
      raw: None,
    }),
    type_only: false,
    with: None,
    phase: Default::default(),
  }))
}

fn const_decl(name: &str, init: Expr) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Const,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(ident(name).into()),
      init: Some(Box::new(init)),
      definite: false,
    }],
    ..Default::default()
  }))))
}

fn const_object_decl(names: &[&str], init: Expr) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Const,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Object(ObjectPat {
        span: DUMMY_SP,
        props: names
          .iter()
          .map(|name| {
            ObjectPatProp::Assign(AssignPatProp {
              span: DUMMY_SP,
              key: ident(name).into(),
              value: None,
            })
          })
          .collect(),
        optional: false,
        type_ann: None,
      }),
      init: Some(Box::new(init)),
      definite: false,
    }],
    ..Default::default()
  }))))
}

fn export_const_decl(name: &str, init: Expr) -> ModuleItem {
  ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
    span: DUMMY_SP,
    decl: Decl::Var(Box::new(VarDecl {
      span: DUMMY_SP,
      kind: VarDeclKind::Const,
      decls: vec![VarDeclarator {
        span: DUMMY_SP,
        name: Pat::Ident(ident(name).into()),
        init: Some(Box::new(init)),
        definite: false,
      }],
      ..Default::default()
    })),
  }))
}

fn export_named_decl(local_name: &str, export_name: &str) -> ModuleItem {
  ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
    span: DUMMY_SP,
    specifiers: vec![ExportSpecifier::Named(ExportNamedSpecifier {
      span: DUMMY_SP,
      orig: ModuleExportName::Ident(ident(local_name)),
      exported: Some(ModuleExportName::Str(Str {
        span: DUMMY_SP,
        value: Wtf8Atom::from(export_name),
        raw: None,
      })),
      is_type_only: false,
    })],
    src: None,
    type_only: false,
    with: None,
  }))
}

fn assign_member_stmt(obj: &str, prop: &str, right: Expr) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Assign(AssignExpr {
      span: DUMMY_SP,
      op: AssignOp::Assign,
      left: AssignTarget::Simple(SimpleAssignTarget::Member(member(obj, prop))),
      right: Box::new(right),
    })),
  }))
}

fn assign_computed_member_stmt(obj: &str, prop: Expr, right: Expr) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Assign(AssignExpr {
      span: DUMMY_SP,
      op: AssignOp::Assign,
      left: AssignTarget::Simple(SimpleAssignTarget::Member(computed_member(
        ident_expr(obj),
        prop,
      ))),
      right: Box::new(right),
    })),
  }))
}

fn require_call(source: &str) -> Expr {
  call_expr(ident_expr("require"), vec![str_expr(source)], DUMMY_SP)
}

fn call_expr(callee: Expr, args: Vec<Expr>, span: Span) -> Expr {
  Expr::Call(CallExpr {
    span,
    callee: callee.as_callee(),
    args: args.into_iter().map(expr_arg).collect(),
    ..Default::default()
  })
}

fn computed_member(obj: Expr, prop: Expr) -> MemberExpr {
  MemberExpr {
    span: DUMMY_SP,
    obj: Box::new(obj),
    prop: MemberProp::Computed(ComputedPropName {
      span: DUMMY_SP,
      expr: Box::new(prop),
    }),
  }
}

fn member(obj: &str, prop: &str) -> MemberExpr {
  member_expr_inner(ident_expr(obj), prop)
}

fn member_expr_inner(obj: Expr, prop: &str) -> MemberExpr {
  MemberExpr {
    span: DUMMY_SP,
    obj: Box::new(obj),
    prop: MemberProp::Ident(ident_name(prop)),
  }
}

fn ident_expr(name: &str) -> Expr {
  Expr::Ident(ident(name))
}

fn str_expr(value: &str) -> Expr {
  Expr::Lit(Lit::Str(Str {
    span: DUMMY_SP,
    value: Wtf8Atom::from(value),
    raw: None,
  }))
}

fn expr_arg(expr: Expr) -> ExprOrSpread {
  ExprOrSpread {
    spread: None,
    expr: Box::new(expr),
  }
}

fn ident(name: &str) -> Ident {
  Ident::new(Atom::from(name), DUMMY_SP, SyntaxContext::empty())
}

fn ident_name(name: &str) -> IdentName {
  IdentName::new(Atom::from(name), DUMMY_SP)
}

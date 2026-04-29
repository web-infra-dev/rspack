use rspack_util::json_stringify_str;
use swc::atoms::Wtf8Atom;
use swc_core::{
  atoms::Atom,
  common::{DUMMY_SP, Span, SyntaxContext},
  ecma::{ast::*, utils::ExprFactory},
};

const RSC_SERVER_MODULE: &str = "react-server-dom-rspack/server";
const REGISTER_CLIENT_REFERENCE: &str = "registerClientReference";
const REACT_MODULE: &str = "react";
const REACT_BINDING: &str = "React";
const REACT_FRAGMENT: &str = "Fragment";
const REACT_CREATE_ELEMENT: &str = "createElement";
const CSS_RESOURCES_BINDING: &str = "resources";
const CLIENT_REF_BINDING_PREFIX: &str = "Ref";
const API_RSC_MANIFEST: &str = "__rspack_rsc_manifest__";
const DATA_RSC_CSS_HREF: &str = "data-rsc-css-href";

/// Replaces a `"use client"` module on the RSC server layer with client
/// reference proxy exports.
///
/// For ESM modules with `default` and `Button` exports, this generates code
/// with this shape:
///
/// ```text
/// import { registerClientReference } from "react-server-dom-rspack/server";
/// import * as React from "react";
///
/// const resources = (__rspack_rsc_manifest__.clientManifest?.[resource]?.cssFiles ?? [])
///   .map(href => React.createElement("link", {
///     key: href,
///     rel: "stylesheet",
///     href,
///     "data-rsc-css-href": href,
///     precedence: "default"
///   }));
///
/// const Ref1 = registerClientReference(function() { throw new Error(...); }, resource, "default");
/// export default resources.length
///   ? props => React.createElement(React.Fragment, null, resources, React.createElement(Ref1, props))
///   : Ref1;
///
/// const Ref2 = registerClientReference(function() { throw new Error(...); }, resource, "Button");
/// export const Button = resources.length
///   ? props => React.createElement(React.Fragment, null, resources, React.createElement(Ref2, props))
///   : Ref2;
/// ```
///
/// CJS modules use the same proxy declarations, but import through `require`
/// and assign to `module.exports` / `exports[exportName]`:
///
/// ```text
/// const { registerClientReference } = require("react-server-dom-rspack/server");
/// const React = require("react");
///
/// const resources = (__rspack_rsc_manifest__.clientManifest?.[resource]?.cssFiles ?? [])
///   .map(href => React.createElement("link", {
///     key: href,
///     rel: "stylesheet",
///     href,
///     "data-rsc-css-href": href,
///     precedence: "default"
///   }));
///
/// const Ref1 = registerClientReference(function() { throw new Error(...); }, resource, "default");
/// module.exports = resources.length
///   ? props => React.createElement(React.Fragment, null, resources, React.createElement(Ref1, props))
///   : Ref1;
///
/// const Ref2 = registerClientReference(function() { throw new Error(...); }, resource, "Button");
/// exports["Button"] = resources.length
///   ? props => React.createElement(React.Fragment, null, resources, React.createElement(Ref2, props))
///   : Ref2;
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
  let mut bindings = BindingNameAllocator::new(client_refs);
  let resources_name = bindings.claim_available(CSS_RESOURCES_BINDING);
  let react_name = bindings.claim_available(REACT_BINDING);

  let mut items = Vec::with_capacity(client_refs.len() * 2 + 3);
  items.push(register_client_reference_decl);
  items.push(react_decl(&react_name, is_cjs));
  items.push(css_resources_decl(resource, &resources_name, &react_name));
  items.extend(client_exports(
    resource,
    client_refs,
    &resources_name,
    &react_name,
    &mut bindings,
    is_cjs,
  ));
  items
}

fn client_exports(
  resource: &str,
  client_refs: &[Wtf8Atom],
  resources_name: &str,
  react_name: &str,
  bindings: &mut BindingNameAllocator,
  is_cjs: bool,
) -> Vec<ModuleItem> {
  let call_error = client_reference_call_error(resource);
  let mut items = Vec::with_capacity(client_refs.len() * 2);

  for export_name in client_refs
    .iter()
    .filter_map(|client_ref| client_ref.as_str())
  {
    let ref_name = bindings.next_ref_name();
    items.push(client_reference_decl(
      resource,
      export_name,
      &ref_name,
      &call_error,
    ));
    items.push(client_export_decl(
      export_name,
      &ref_name,
      resources_name,
      react_name,
      is_cjs,
    ));
  }

  items
}

fn client_export_decl(
  export_name: &str,
  ref_name: &str,
  resources_name: &str,
  react_name: &str,
  is_cjs: bool,
) -> ModuleItem {
  let export_expr = client_export_expr(ref_name, resources_name, react_name);

  match (is_cjs, export_name) {
    (false, "default") => {
      ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
        span: DUMMY_SP,
        expr: Box::new(export_expr),
      }))
    }
    (true, "default") => assign_member_stmt("module", "exports", export_expr),
    (false, ident) => export_const_decl(ident, export_expr),
    (true, ident) => assign_computed_member_stmt("exports", str_expr(ident), export_expr),
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

fn client_reference_decl(
  resource: &str,
  export_name: &str,
  ref_name: &str,
  call_error: &str,
) -> ModuleItem {
  const_decl(
    ref_name,
    register_client_reference_expr(resource, export_name, call_error),
  )
}

struct BindingNameAllocator<'a> {
  client_refs: &'a [Wtf8Atom],
  reserved_names: Vec<String>,
  ref_count: usize,
}

impl<'a> BindingNameAllocator<'a> {
  fn new(client_refs: &'a [Wtf8Atom]) -> Self {
    Self {
      client_refs,
      reserved_names: Vec::new(),
      ref_count: 0,
    }
  }

  fn claim_available(&mut self, base: &str) -> String {
    let mut suffix = 0;
    loop {
      let name = if suffix == 0 {
        base.to_string()
      } else {
        format!("{base}{suffix}")
      };

      if self.is_available(&name) {
        self.reserved_names.push(name.clone());
        return name;
      }

      suffix += 1;
    }
  }

  fn next_ref_name(&mut self) -> String {
    loop {
      self.ref_count += 1;
      let name = format!("{CLIENT_REF_BINDING_PREFIX}{}", self.ref_count);

      if self.is_available(&name) {
        self.reserved_names.push(name.clone());
        return name;
      }
    }
  }

  fn is_available(&self, name: &str) -> bool {
    !self
      .reserved_names
      .iter()
      .any(|reserved_name| reserved_name == name)
      && !self
        .client_refs
        .iter()
        .any(|client_ref| client_ref.as_str() == Some(name))
  }
}

fn css_resources_decl(resource: &str, resources_name: &str, react_name: &str) -> ModuleItem {
  const_decl(
    resources_name,
    call_expr(
      member_expr(
        nullish_coalescing(client_css_files_expr(resource), empty_array_expr()),
        "map",
      ),
      vec![css_resource_mapper(react_name)],
      DUMMY_SP,
    ),
  )
}

fn empty_array_expr() -> Expr {
  Expr::Array(ArrayLit {
    span: DUMMY_SP,
    elems: vec![],
  })
}

fn client_css_files_expr(resource: &str) -> Expr {
  opt_chain_member_expr(
    opt_chain_computed_member_expr(client_manifest_expr(), str_expr(resource)),
    "cssFiles",
  )
}

fn client_manifest_expr() -> Expr {
  member_expr(ident_expr(API_RSC_MANIFEST), "clientManifest")
}

fn css_resource_mapper(react_name: &str) -> Expr {
  Expr::Fn(FnExpr {
    ident: None,
    function: Box::new(Function {
      params: vec![Param {
        span: DUMMY_SP,
        decorators: vec![],
        pat: Pat::Ident(ident_name("href").into()),
      }],
      body: Some(BlockStmt {
        span: DUMMY_SP,
        stmts: vec![Stmt::Return(ReturnStmt {
          span: DUMMY_SP,
          arg: Some(Box::new(react_link_element(react_name))),
        })],
        ..Default::default()
      }),
      ..Default::default()
    }),
  })
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

fn client_export_expr(ref_name: &str, resources_name: &str, react_name: &str) -> Expr {
  Expr::Cond(CondExpr {
    span: DUMMY_SP,
    test: Box::new(member_expr(ident_expr(resources_name), "length")),
    cons: Box::new(client_wrapper_arrow(ref_name, resources_name, react_name)),
    alt: Box::new(ident_expr(ref_name)),
  })
}

fn client_wrapper_arrow(ref_name: &str, resources_name: &str, react_name: &str) -> Expr {
  arrow_expr(
    &["props"],
    react_fragment_with_resources(ref_name, resources_name, react_name),
  )
}

fn react_fragment_with_resources(ref_name: &str, resources_name: &str, react_name: &str) -> Expr {
  react_create_element_call(
    react_name,
    member_expr(ident_expr(react_name), REACT_FRAGMENT),
    null_expr(),
    vec![
      ident_expr(resources_name),
      react_create_element_call(
        react_name,
        ident_expr(ref_name),
        ident_expr("props"),
        vec![],
      ),
    ],
  )
}

fn react_link_element(react_name: &str) -> Expr {
  // Mark these stylesheet links as RSC-managed CSS dependencies. Consumers can
  // collect this marker and preinit only bundler-emitted RSC CSS, without
  // treating arbitrary user-authored stylesheet links as RSC assets.
  react_create_element_call(
    react_name,
    str_expr("link"),
    object_expr(vec![
      key_value_prop("key", ident_expr("href")),
      key_value_prop("rel", str_expr("stylesheet")),
      key_value_prop("href", ident_expr("href")),
      key_value_str_prop(DATA_RSC_CSS_HREF, ident_expr("href")),
      key_value_prop("precedence", str_expr("default")),
    ]),
    vec![],
  )
}

fn react_create_element_call(
  react_name: &str,
  element: Expr,
  props: Expr,
  children: Vec<Expr>,
) -> Expr {
  let mut args = vec![element, props];
  args.extend(children);
  call_expr(
    member_expr(ident_expr(react_name), REACT_CREATE_ELEMENT),
    args,
    DUMMY_SP,
  )
}

fn object_expr(props: Vec<PropOrSpread>) -> Expr {
  Expr::Object(ObjectLit {
    span: DUMMY_SP,
    props,
  })
}

fn key_value_prop(name: &str, value: Expr) -> PropOrSpread {
  PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
    key: PropName::Ident(ident_name(name)),
    value: Box::new(value),
  })))
}

fn key_value_str_prop(name: &str, value: Expr) -> PropOrSpread {
  PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
    key: PropName::Str(Str {
      span: DUMMY_SP,
      value: Wtf8Atom::from(name),
      raw: None,
    }),
    value: Box::new(value),
  })))
}

fn null_expr() -> Expr {
  Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))
}

fn react_decl(name: &str, is_cjs: bool) -> ModuleItem {
  if is_cjs {
    const_decl(name, require_call(REACT_MODULE))
  } else {
    import_namespace(REACT_MODULE, name)
  }
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

fn import_namespace(source: &str, name: &str) -> ModuleItem {
  ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
    span: DUMMY_SP,
    specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
      span: DUMMY_SP,
      local: ident(name),
    })],
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

fn arrow_expr(params: &[&str], body: Expr) -> Expr {
  Expr::Arrow(ArrowExpr {
    span: DUMMY_SP,
    ctxt: SyntaxContext::empty(),
    params: params
      .iter()
      .map(|name| Pat::Ident(ident_name(name).into()))
      .collect(),
    body: Box::new(BlockStmtOrExpr::Expr(Box::new(body))),
    is_async: false,
    is_generator: false,
    type_params: None,
    return_type: None,
  })
}

fn call_expr(callee: Expr, args: Vec<Expr>, span: Span) -> Expr {
  Expr::Call(CallExpr {
    span,
    callee: callee.as_callee(),
    args: args.into_iter().map(expr_arg).collect(),
    ..Default::default()
  })
}

fn nullish_coalescing(left: Expr, right: Expr) -> Expr {
  Expr::Bin(BinExpr {
    span: DUMMY_SP,
    op: BinaryOp::NullishCoalescing,
    left: Box::new(left),
    right: Box::new(right),
  })
}

fn opt_chain_computed_member_expr(obj: Expr, prop: Expr) -> Expr {
  opt_chain_member(computed_member(obj, prop))
}

fn opt_chain_member_expr(obj: Expr, prop: &str) -> Expr {
  opt_chain_member(member_expr_inner(obj, prop))
}

fn opt_chain_member(member: MemberExpr) -> Expr {
  Expr::OptChain(OptChainExpr {
    span: DUMMY_SP,
    base: Box::new(OptChainBase::Member(member)),
    optional: true,
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

fn member_expr(obj: Expr, prop: &str) -> Expr {
  Expr::Member(member_expr_inner(obj, prop))
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

use std::{
  cell::RefCell,
  convert::{TryFrom, TryInto},
  mem::{replace, take},
};

use indoc::formatdoc;
use rspack_core::{RscMeta, RscModuleType};
use rspack_util::fx_hash::FxIndexMap;
use rustc_hash::FxHashSet;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use swc_core::{
  atoms::{Atom, Wtf8Atom, atom},
  common::{
    BytePos, DUMMY_SP, Mark, Span, SyntaxContext, comments::Comments, errors::HANDLER,
    source_map::PURE_SP, util::take::Take,
  },
  ecma::{
    ast::*,
    utils::{ExprFactory, private_ident, quote_ident},
    visit::{VisitMut, VisitMutWith, noop_visit_mut_type, visit_mut_pass},
  },
};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Config {
  pub is_react_server_layer: bool,
  pub is_development: bool,
  pub hash_salt: String,
}

#[derive(Clone, Debug)]
enum DirectiveLocation {
  Module,
  FunctionBody,
}

#[derive(Clone, Debug)]
enum ThisStatus {
  Allowed,
  Forbidden,
}

#[derive(Clone)]
struct ServerReferenceExport {
  ident: Ident,
  export_name: ModuleExportName,
  reference_id: Atom,
}

#[derive(Clone, Debug)]
enum ServerActionsErrorKind {
  ExportedSyncFunction {
    span: Span,
  },
  ForbiddenExpression {
    span: Span,
    expr: String,
  },
  InlineSyncFunction {
    span: Span,
  },
  InlineUseServerInClassInstanceMethod {
    span: Span,
  },
  InlineUseServerInClientComponent {
    span: Span,
  },
  MisplacedDirective {
    span: Span,
    directive: String,
    location: DirectiveLocation,
  },
  MisplacedWrappedDirective {
    span: Span,
    directive: String,
    location: DirectiveLocation,
  },
  MisspelledDirective {
    span: Span,
    directive: String,
    expected_directive: String,
  },
  MultipleDirectives {
    span: Span,
    location: DirectiveLocation,
  },
  WrappedDirective {
    span: Span,
    directive: String,
  },
}

pub fn server_actions<C: Comments>(
  file_name: String,
  config: Config,
  comments: C,
  rsc_meta: &RefCell<Option<RscMeta>>,
) -> impl Pass {
  visit_mut_pass(ServerActions {
    config,
    comments,
    rsc_meta,
    file_name,
    start_pos: BytePos(0),
    in_action_file: false,
    current_export_name: None,
    fn_decl_ident: None,
    in_callee: false,
    has_action: false,
    this_status: ThisStatus::Allowed,

    reference_index: 0,
    in_module_level: true,
    should_track_names: false,
    has_server_reference_with_bound_args: false,

    names: Default::default(),
    declared_idents: Default::default(),

    // This flag allows us to rewrite `function foo() {}` to `const foo = createProxy(...)`.
    rewrite_fn_decl_to_proxy_decl: None,
    rewrite_default_fn_expr_to_proxy_expr: None,
    rewrite_expr_to_proxy_expr: None,

    annotations: Default::default(),
    extra_items: Default::default(),
    hoisted_extra_items: Default::default(),
    reference_ids_by_export_name: Default::default(),
    server_reference_exports: Default::default(),

    private_ctxt: SyntaxContext::empty().apply_mark(Mark::new()),

    arrow_or_fn_expr_ident: None,
    export_name_by_local_id: Default::default(),
    local_ids_that_need_cache_runtime_wrapper_if_exported: FxHashSet::default(),
  })
}

struct ServerActions<'a, C: Comments> {
  #[allow(unused)]
  config: Config,
  file_name: String,
  comments: C,
  rsc_meta: &'a RefCell<Option<RscMeta>>,

  start_pos: BytePos,
  in_action_file: bool,
  current_export_name: Option<ModuleExportName>,
  fn_decl_ident: Option<Ident>,
  in_callee: bool,
  has_action: bool,
  this_status: ThisStatus,

  reference_index: u32,
  in_module_level: bool,
  should_track_names: bool,
  has_server_reference_with_bound_args: bool,

  names: Vec<Name>,
  declared_idents: Vec<Ident>,

  // This flag allows us to rewrite `function foo() {}` to `const foo = createProxy(...)`.
  rewrite_fn_decl_to_proxy_decl: Option<VarDecl>,
  rewrite_default_fn_expr_to_proxy_expr: Option<Box<Expr>>,
  rewrite_expr_to_proxy_expr: Option<Box<Expr>>,

  annotations: Vec<Stmt>,
  extra_items: Vec<ModuleItem>,
  hoisted_extra_items: Vec<ModuleItem>,

  /// A map of all server references (inline + exported): export_name -> reference_id
  reference_ids_by_export_name: FxIndexMap<ModuleExportName, Atom>,

  /// A list of server references for originally exported server functions only.
  server_reference_exports: Vec<ServerReferenceExport>,

  private_ctxt: SyntaxContext,

  arrow_or_fn_expr_ident: Option<Ident>,
  export_name_by_local_id: FxIndexMap<Id, ModuleExportName>,

  /// Tracks which local IDs need cache runtime wrappers if exported (collected during pre-pass).
  /// Includes imports, destructured identifiers, and variables with unknown-type init
  /// expressions (calls, identifiers, etc.). Excludes known functions (arrow/fn
  /// declarations) and known non-functions (object/array/literals). When these IDs are
  /// exported, their exports are stripped and replaced with conditional wrappers.
  local_ids_that_need_cache_runtime_wrapper_if_exported: FxHashSet<Id>,
}

impl<'a, C: Comments> ServerActions<'a, C> {
  fn generate_server_reference_id(
    &self,
    export_name: &ModuleExportName,
    params: Option<&Vec<Param>>,
  ) -> Atom {
    // Attach a checksum to the action using sha1:
    // $$id = special_byte + sha1('hash_salt' + 'file_name' + ':' + 'export_name');
    // Currently encoded as hex.

    let mut hasher = Sha256::new();
    hasher.update(self.config.hash_salt.as_bytes());
    hasher.update(self.file_name.as_bytes());
    hasher.update(b":");

    let export_name_bytes = match export_name {
      ModuleExportName::Ident(ident) => &ident.sym.as_bytes(),
      ModuleExportName::Str(s) => &s.value.as_bytes(),
    };

    hasher.update(export_name_bytes);

    let mut result = hasher.finalize().to_vec();

    // Prepend an extra byte to the ID, with the following format:
    // 0     000000    0
    // ^type ^arg mask ^rest args
    //
    // The type bit represents if the action is a cache function or not.
    // For cache functions, the type bit is set to 1. Otherwise, it's 0.
    //
    // The arg mask bit is used to determine which arguments are used by
    // the function itself, up to 6 arguments. The bit is set to 1 if the
    // argument is used, or being spread or destructured (so it can be
    // indirectly or partially used). The bit is set to 0 otherwise.
    //
    // The rest args bit is used to determine if there's a ...rest argument
    // in the function signature. If there is, the bit is set to 1.
    //
    //  For example:
    //
    //   async function foo(a, foo, b, bar, ...baz) {
    //     'use cache';
    //     return a + b;
    //   }
    //
    // will have it encoded as [1][101011][1]. The first bit is set to 1
    // because it's a cache function. The second part has 1010 because the
    // only arguments used are `a` and `b`. The subsequent 11 bits are set
    // to 1 because there's a ...rest argument starting from the 5th. The
    // last bit is set to 1 as well for the same reason.
    let type_bit = 0u8;
    let mut arg_mask = 0u8;
    let mut rest_args = 0u8;

    if let Some(params) = params {
      // TODO: For the current implementation, we don't track if an
      // argument ident is actually referenced in the function body.
      // Instead, we go with the easy route and assume defined ones are
      // used. This can be improved in the future.
      for (i, param) in params.iter().enumerate() {
        if let Pat::Rest(_) = param.pat {
          // If there's a ...rest argument, we set the rest args bit
          // to 1 and set the arg mask to 0b111111.
          arg_mask = 0b111111;
          rest_args = 0b1;
          break;
        }
        if i < 6 {
          arg_mask |= 0b1 << (5 - i);
        } else {
          // More than 6 arguments, we set the rest args bit to 1.
          // This is rare for a Server Action, usually.
          rest_args = 0b1;
          break;
        }
      }
    } else {
      // If we can't determine the arguments (e.g. not statically analyzable),
      // we assume all arguments are used.
      arg_mask = 0b111111;
      rest_args = 0b1;
    }

    result.push((type_bit << 7) | (arg_mask << 1) | rest_args);
    result.rotate_right(1);

    Atom::from(hex::encode(result))
  }

  fn is_default_export(&self) -> bool {
    matches!(
        self.current_export_name,
        Some(ModuleExportName::Ident(ref i)) if i.sym == *"default"
    )
  }

  fn gen_action_ident(&mut self) -> Atom {
    let id: Atom = format!("$$RSC_SERVER_ACTION_{0}", self.reference_index).into();
    self.reference_index += 1;
    id
  }

  fn create_bound_action_args_array_pat(&mut self, arg_len: usize) -> Pat {
    Pat::Array(ArrayPat {
      span: DUMMY_SP,
      elems: (0..arg_len)
        .map(|i| {
          Some(Pat::Ident(
            Ident::new(
              format!("$$ACTION_ARG_{i}").into(),
              DUMMY_SP,
              self.private_ctxt,
            )
            .into(),
          ))
        })
        .collect(),
      optional: false,
      type_ann: None,
    })
  }

  // Check if the function or arrow function is an action function,
  // and remove any server function directive.
  fn has_use_server_for_function(&mut self, maybe_body: Option<&mut BlockStmt>) -> bool {
    let mut found_use_server = false;

    // Even if it's a file-level action or cache module, the function body
    // might still have directives that override the module-level annotations.
    if let Some(body) = maybe_body {
      let directive_visitor = &mut DirectiveVisitor {
        config: &self.config,
        found_use_server: false,
        in_action_file: self.in_action_file,
        is_allowed_position: true,
        location: DirectiveLocation::FunctionBody,
      };

      body.stmts.retain(|stmt| {
        let has_directive = directive_visitor.visit_stmt(stmt);

        !has_directive
      });

      found_use_server = directive_visitor.found_use_server;
    }

    // All exported functions inherit the file directive if they don't have their own directive.
    if self.current_export_name.is_some() && !found_use_server && self.in_action_file {
      return true;
    }

    found_use_server
  }

  fn has_use_server_for_module(&mut self, stmts: &mut Vec<ModuleItem>) -> bool {
    let directive_visitor = &mut DirectiveVisitor {
      config: &self.config,
      found_use_server: false,
      in_action_file: false,
      is_allowed_position: true,
      location: DirectiveLocation::Module,
    };

    stmts.retain(|item| {
      if let ModuleItem::Stmt(stmt) = item {
        let has_directive = directive_visitor.visit_stmt(stmt);

        !has_directive
      } else {
        directive_visitor.is_allowed_position = false;
        true
      }
    });

    directive_visitor.found_use_server
  }

  fn maybe_hoist_and_create_proxy_for_server_action_arrow_expr(
    &mut self,
    ids_from_closure: Vec<Name>,
    arrow: &mut ArrowExpr,
  ) -> Box<Expr> {
    let mut new_params: Vec<Param> = vec![];

    if !ids_from_closure.is_empty() {
      // First param is the encrypted closure variables.
      new_params.push(Param {
        span: DUMMY_SP,
        decorators: vec![],
        pat: Pat::Ident(IdentName::new(atom!("$$ACTION_CLOSURE_BOUND"), DUMMY_SP).into()),
      });
    }

    for p in arrow.params.iter() {
      new_params.push(Param::from(p.clone()));
    }

    let action_name = self.gen_action_ident();
    let action_ident = Ident::new(action_name.clone(), arrow.span, self.private_ctxt);
    let action_id = self.generate_server_reference_id(
      &ModuleExportName::Ident(action_ident.clone()),
      Some(&new_params),
    );

    self.has_action = true;
    self.reference_ids_by_export_name.insert(
      ModuleExportName::Ident(action_ident.clone()),
      action_id.clone(),
    );

    // If this is an exported arrow, remove it from export_name_by_local_id so the
    // post-pass doesn't register it again (it's already registered above).
    if self.current_export_name.is_some()
      && let Some(arrow_ident) = &self.arrow_or_fn_expr_ident
    {
      self
        .export_name_by_local_id
        .swap_remove(&arrow_ident.to_id());
    }

    if let BlockStmtOrExpr::BlockStmt(block) = &mut *arrow.body {
      block.visit_mut_with(&mut ClosureReplacer {
        used_ids: &ids_from_closure,
        private_ctxt: self.private_ctxt,
      });
    }

    let mut new_body: BlockStmtOrExpr = *arrow.body.clone();

    if !ids_from_closure.is_empty() {
      // Prepend the decryption declaration to the body.
      // var [arg1, arg2, arg3] = await decryptActionBoundArgs(actionId,
      // $$ACTION_CLOSURE_BOUND)
      let decryption_decl = VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
          span: DUMMY_SP,
          name: self.create_bound_action_args_array_pat(ids_from_closure.len()),
          init: Some(Box::new(Expr::Await(AwaitExpr {
            span: DUMMY_SP,
            arg: Box::new(Expr::Call(CallExpr {
              span: DUMMY_SP,
              callee: quote_ident!("decryptActionBoundArgs").as_callee(),
              args: vec![
                action_id.clone().as_arg(),
                quote_ident!("$$ACTION_CLOSURE_BOUND").as_arg(),
              ],
              ..Default::default()
            })),
          }))),
          definite: Default::default(),
        }],
        ..Default::default()
      };

      match &mut new_body {
        BlockStmtOrExpr::BlockStmt(body) => {
          body.stmts.insert(0, decryption_decl.into());
        }
        BlockStmtOrExpr::Expr(body_expr) => {
          new_body = BlockStmtOrExpr::BlockStmt(BlockStmt {
            span: DUMMY_SP,
            stmts: vec![
              decryption_decl.into(),
              Stmt::Return(ReturnStmt {
                span: DUMMY_SP,
                arg: Some(body_expr.take()),
              }),
            ],
            ..Default::default()
          });
        }
      }
    }

    // Create the action export decl from the arrow function
    // export const $$RSC_SERVER_ACTION_0 = async function action($$ACTION_CLOSURE_BOUND) {}
    self
      .hoisted_extra_items
      .push(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
        span: DUMMY_SP,
        decl: VarDecl {
          kind: VarDeclKind::Const,
          span: DUMMY_SP,
          decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(action_ident.clone().into()),
            definite: false,
            init: Some(Box::new(Expr::Fn(FnExpr {
              ident: self.arrow_or_fn_expr_ident.clone(),
              function: Box::new(Function {
                params: new_params,
                body: match new_body {
                  BlockStmtOrExpr::BlockStmt(body) => Some(body),
                  BlockStmtOrExpr::Expr(expr) => Some(BlockStmt {
                    span: DUMMY_SP,
                    stmts: vec![Stmt::Return(ReturnStmt {
                      span: DUMMY_SP,
                      arg: Some(expr),
                    })],
                    ..Default::default()
                  }),
                },
                is_async: true,
                ..Default::default()
              }),
            }))),
          }],
          declare: Default::default(),
          ctxt: self.private_ctxt,
        }
        .into(),
      })));

    self
      .hoisted_extra_items
      .push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(annotate_ident_as_server_reference(
          action_ident.clone(),
          action_id.clone(),
          arrow.span,
        )),
      })));

    if ids_from_closure.is_empty() {
      Box::new(action_ident.clone().into())
    } else {
      self.has_server_reference_with_bound_args = true;
      Box::new(bind_args_to_ident(
        action_ident.clone(),
        ids_from_closure
          .iter()
          .cloned()
          .map(|id| Some(id.as_arg()))
          .collect(),
        action_id.clone(),
      ))
    }
  }

  fn maybe_hoist_and_create_proxy_for_server_action_function(
    &mut self,
    ids_from_closure: Vec<Name>,
    function: &mut Function,
    fn_name: Option<Ident>,
  ) -> Box<Expr> {
    let mut new_params: Vec<Param> = vec![];

    if !ids_from_closure.is_empty() {
      // First param is the encrypted closure variables.
      new_params.push(Param {
        span: DUMMY_SP,
        decorators: vec![],
        pat: Pat::Ident(IdentName::new(atom!("$$ACTION_CLOSURE_BOUND"), DUMMY_SP).into()),
      });
    }

    new_params.append(&mut function.params);

    let action_name: Atom = self.gen_action_ident();
    let mut action_ident = Ident::new(action_name.clone(), function.span, self.private_ctxt);
    if action_ident.span.lo == self.start_pos {
      action_ident.span = Span::dummy_with_cmt();
    }

    let action_id = self.generate_server_reference_id(
      &ModuleExportName::Ident(action_ident.clone()),
      Some(&new_params),
    );

    self.has_action = true;
    self.reference_ids_by_export_name.insert(
      ModuleExportName::Ident(action_ident.clone()),
      action_id.clone(),
    );

    // If this is an exported function, remove it from export_name_by_local_id so the
    // post-pass doesn't register it again (it's already registered above).
    if self.current_export_name.is_some()
      && let Some(ref fn_name) = fn_name
    {
      self.export_name_by_local_id.swap_remove(&fn_name.to_id());
    }

    function.body.visit_mut_with(&mut ClosureReplacer {
      used_ids: &ids_from_closure,
      private_ctxt: self.private_ctxt,
    });

    let mut new_body: Option<BlockStmt> = function.body.clone();

    if !ids_from_closure.is_empty() {
      // Prepend the decryption declaration to the body.
      // var [arg1, arg2, arg3] = await decryptActionBoundArgs(actionId,
      // $$ACTION_CLOSURE_BOUND)
      let decryption_decl = VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Var,
        decls: vec![VarDeclarator {
          span: DUMMY_SP,
          name: self.create_bound_action_args_array_pat(ids_from_closure.len()),
          init: Some(Box::new(Expr::Await(AwaitExpr {
            span: DUMMY_SP,
            arg: Box::new(Expr::Call(CallExpr {
              span: DUMMY_SP,
              callee: quote_ident!("decryptActionBoundArgs").as_callee(),
              args: vec![
                action_id.clone().as_arg(),
                quote_ident!("$$ACTION_CLOSURE_BOUND").as_arg(),
              ],
              ..Default::default()
            })),
          }))),
          definite: Default::default(),
        }],
        ..Default::default()
      };

      if let Some(body) = &mut new_body {
        body.stmts.insert(0, decryption_decl.into());
      } else {
        new_body = Some(BlockStmt {
          span: DUMMY_SP,
          stmts: vec![decryption_decl.into()],
          ..Default::default()
        });
      }
    }

    // Create the action export decl from the function
    // export const $$RSC_SERVER_ACTION_0 = async function action($$ACTION_CLOSURE_BOUND) {}
    self
      .hoisted_extra_items
      .push(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
        span: DUMMY_SP,
        decl: VarDecl {
          kind: VarDeclKind::Const,
          span: DUMMY_SP,
          decls: vec![VarDeclarator {
            span: DUMMY_SP, // TODO: need to map it to the original span?
            name: Pat::Ident(action_ident.clone().into()),
            definite: false,
            init: Some(Box::new(Expr::Fn(FnExpr {
              ident: fn_name,
              function: Box::new(Function {
                params: new_params,
                body: new_body,
                ..function.take()
              }),
            }))),
          }],
          declare: Default::default(),
          ctxt: self.private_ctxt,
        }
        .into(),
      })));

    self
      .hoisted_extra_items
      .push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(annotate_ident_as_server_reference(
          action_ident.clone(),
          action_id.clone(),
          function.span,
        )),
      })));

    if ids_from_closure.is_empty() {
      Box::new(action_ident.clone().into())
    } else {
      self.has_server_reference_with_bound_args = true;
      Box::new(bind_args_to_ident(
        action_ident.clone(),
        ids_from_closure
          .iter()
          .cloned()
          .map(|id| Some(id.as_arg()))
          .collect(),
        action_id.clone(),
      ))
    }
  }

  /// Validates that a function is async, emitting an error if not.
  /// Returns true if async, false otherwise.
  fn validate_async_function(&self, is_async: bool, span: Span, fn_name: Option<&Ident>) -> bool {
    if is_async {
      true
    } else {
      emit_error(ServerActionsErrorKind::InlineSyncFunction {
        span: fn_name.as_ref().map_or(span, |ident| ident.span),
      });
      false
    }
  }

  /// Registers a server action export (for a 'use server' file directive).
  fn register_server_action_export(
    &mut self,
    export_name: &ModuleExportName,
    fn_name: Option<&Ident>,
    params: Option<&Vec<Param>>,
    span: Span,
    take_fn_or_arrow_expr: &mut dyn FnMut() -> Box<Expr>,
  ) {
    if let Some(fn_name) = fn_name {
      let reference_id = self.generate_server_reference_id(export_name, params);

      self.has_action = true;
      self
        .reference_ids_by_export_name
        .insert(export_name.clone(), reference_id.clone());

      self.server_reference_exports.push(ServerReferenceExport {
        ident: fn_name.clone(),
        export_name: export_name.clone(),
        reference_id: reference_id.clone(),
      });
    } else if self.is_default_export() {
      let action_ident = Ident::new(self.gen_action_ident(), span, self.private_ctxt);
      let reference_id = self.generate_server_reference_id(export_name, params);

      self.has_action = true;
      self
        .reference_ids_by_export_name
        .insert(export_name.clone(), reference_id.clone());

      self.server_reference_exports.push(ServerReferenceExport {
        ident: action_ident.clone(),
        export_name: export_name.clone(),
        reference_id: reference_id.clone(),
      });

      // For the server layer, also hoist the function and rewrite the default export.
      if self.config.is_react_server_layer {
        self
          .hoisted_extra_items
          .push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
            kind: VarDeclKind::Const,
            decls: vec![VarDeclarator {
              span: DUMMY_SP,
              name: Pat::Ident(action_ident.clone().into()),
              init: Some(take_fn_or_arrow_expr()),
              definite: false,
            }],
            ..Default::default()
          })))));

        self
          .hoisted_extra_items
          .push(ModuleItem::Stmt(assign_name_to_ident(
            &action_ident,
            "default",
          )));

        self.rewrite_default_fn_expr_to_proxy_expr = Some(Box::new(Expr::Ident(action_ident)));
      }
    }
  }

  fn set_action_ids(
    &self,
    export_names_ordered_by_reference_id: &FxIndexMap<&Atom, &ModuleExportName>,
  ) {
    let action_ids = export_names_ordered_by_reference_id
      .iter()
      .map(|(ref_id, export_name)| ((**ref_id).clone(), export_name.atom().into_owned()))
      .collect::<FxIndexMap<_, _>>();

    let mut rsc_meta = self.rsc_meta.borrow_mut();
    match rsc_meta.as_mut() {
      Some(rsc_meta) => {
        rsc_meta.action_ids = action_ids;
      }
      None => {
        *rsc_meta = Some(RscMeta {
          module_type: RscModuleType::Server,
          server_refs: Default::default(),
          client_refs: Default::default(),
          is_cjs: false,
          action_ids,
        });
      }
    }
  }
}

impl<'a, C: Comments> VisitMut for ServerActions<'a, C> {
  fn visit_mut_export_decl(&mut self, decl: &mut ExportDecl) {
    // For inline exports like `export function foo() {}` or `export const bar = ...`,
    // the export name is looked up from export_name_by_local_id and set as current_export_name
    // in visit_mut_fn_decl or visit_mut_var_declarator.
    decl.decl.visit_mut_with(self);
  }

  fn visit_mut_export_default_decl(&mut self, decl: &mut ExportDefaultDecl) {
    let old_current_export_name = self.current_export_name.take();
    self.current_export_name = Some(ModuleExportName::Ident(atom!("default").into()));
    self.rewrite_default_fn_expr_to_proxy_expr = None;
    decl.decl.visit_mut_with(self);
    self.current_export_name = old_current_export_name;
  }

  fn visit_mut_export_default_expr(&mut self, expr: &mut ExportDefaultExpr) {
    let old_current_export_name = self.current_export_name.take();
    self.current_export_name = Some(ModuleExportName::Ident(atom!("default").into()));
    expr.expr.visit_mut_with(self);
    self.current_export_name = old_current_export_name;

    // For 'use server' files with call expressions as default exports,
    // hoist the call expression to a const declarator.
    if matches!(&*expr.expr, Expr::Call(_)) && self.in_action_file {
      let export_name = ModuleExportName::Ident(atom!("default").into());
      let action_ident = Ident::new(self.gen_action_ident(), expr.span, self.private_ctxt);
      let action_id = self.generate_server_reference_id(&export_name, None);

      self.has_action = true;
      self
        .reference_ids_by_export_name
        .insert(export_name.clone(), action_id.clone());

      self.server_reference_exports.push(ServerReferenceExport {
        ident: action_ident.clone(),
        export_name: export_name.clone(),
        reference_id: action_id.clone(),
      });

      self
        .hoisted_extra_items
        .push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
          kind: VarDeclKind::Const,
          decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(action_ident.clone().into()),
            init: Some(expr.expr.take()),
            definite: false,
          }],
          ..Default::default()
        })))));

      self.rewrite_default_fn_expr_to_proxy_expr = Some(Box::new(Expr::Ident(action_ident)));
    }
  }

  fn visit_mut_fn_expr(&mut self, f: &mut FnExpr) {
    let old_this_status = replace(&mut self.this_status, ThisStatus::Allowed);
    let old_arrow_or_fn_expr_ident = self.arrow_or_fn_expr_ident.clone();
    if let Some(ident) = &f.ident {
      self.arrow_or_fn_expr_ident = Some(ident.clone());
    }
    f.visit_mut_children_with(self);
    self.this_status = old_this_status;
    self.arrow_or_fn_expr_ident = old_arrow_or_fn_expr_ident;
  }

  fn visit_mut_function(&mut self, f: &mut Function) {
    let found_use_server = self.has_use_server_for_function(f.body.as_mut());
    let declared_idents_until = self.declared_idents.len();
    let old_names = take(&mut self.names);

    if found_use_server {
      self.this_status = ThisStatus::Forbidden;
    }

    // Visit children
    {
      let old_in_module = replace(&mut self.in_module_level, false);
      let should_track_names = found_use_server || self.should_track_names;
      let old_should_track_names = replace(&mut self.should_track_names, should_track_names);
      let old_current_export_name = self.current_export_name.take();
      let old_fn_decl_ident = self.fn_decl_ident.take();
      f.visit_mut_children_with(self);
      self.in_module_level = old_in_module;
      self.should_track_names = old_should_track_names;
      self.current_export_name = old_current_export_name;
      self.fn_decl_ident = old_fn_decl_ident;
    }

    let mut child_names = take(&mut self.names);

    if self.should_track_names {
      self.names = [old_names, child_names.clone()].concat();
    }

    if found_use_server {
      let fn_name = self
        .fn_decl_ident
        .as_ref()
        .or(self.arrow_or_fn_expr_ident.as_ref())
        .cloned();

      if !self.validate_async_function(f.is_async, f.span, fn_name.as_ref()) {
        // If this is an exported function that failed validation, remove it from
        // export_name_by_local_id so the post-pass doesn't register it.
        if self.current_export_name.is_some()
          && let Some(fn_name) = fn_name
        {
          self.export_name_by_local_id.swap_remove(&fn_name.to_id());
        }

        return;
      }

      // If this function is invalid, or any prior errors have been emitted, skip further
      // processing.
      if HANDLER.with(|handler| handler.has_errors()) {
        return;
      }

      // For server action files, register exports without hoisting (for both server and
      // client layers).
      if self.in_action_file
        && found_use_server
        && let Some(export_name) = self.current_export_name.clone()
      {
        let params = f.params.clone();
        let span = f.span;

        self.register_server_action_export(
          &export_name,
          fn_name.as_ref(),
          Some(&params),
          span,
          &mut || {
            Box::new(Expr::Fn(FnExpr {
              ident: fn_name.clone(),
              function: Box::new(f.take()),
            }))
          },
        );

        return;
      }

      // Collect all the identifiers defined inside the closure and used
      // in the action function. With deduplication.
      retain_names_from_declared_idents(
        &mut child_names,
        &self.declared_idents[..declared_idents_until],
      );

      let new_expr =
        self.maybe_hoist_and_create_proxy_for_server_action_function(child_names, f, fn_name);

      if self.is_default_export() {
        // This function expression is also the default export:
        // `export default async function() {}`
        // This specific case (default export) isn't handled by `visit_mut_expr`.
        // Replace the original function expr with a action proxy expr.
        self.rewrite_default_fn_expr_to_proxy_expr = Some(new_expr);
      } else if let Some(ident) = &self.fn_decl_ident {
        // Replace the original function declaration with an action proxy
        // declaration expr.
        self.rewrite_fn_decl_to_proxy_decl = Some(VarDecl {
          span: DUMMY_SP,
          kind: VarDeclKind::Var,
          decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(ident.clone().into()),
            init: Some(new_expr),
            definite: false,
          }],
          ..Default::default()
        });
      } else {
        self.rewrite_expr_to_proxy_expr = Some(new_expr);
      }
    }
  }

  fn visit_mut_decl(&mut self, d: &mut Decl) {
    self.rewrite_fn_decl_to_proxy_decl = None;
    d.visit_mut_children_with(self);

    if let Some(decl) = &self.rewrite_fn_decl_to_proxy_decl {
      *d = (*decl).clone().into();
    }

    self.rewrite_fn_decl_to_proxy_decl = None;
  }

  fn visit_mut_fn_decl(&mut self, f: &mut FnDecl) {
    let old_this_status = replace(&mut self.this_status, ThisStatus::Allowed);
    let old_current_export_name = self.current_export_name.take();
    if self.in_module_level
      && let Some(export_name) = self.export_name_by_local_id.get(&f.ident.to_id())
    {
      self.current_export_name = Some(export_name.clone());
    }
    let old_fn_decl_ident = self.fn_decl_ident.replace(f.ident.clone());
    f.visit_mut_children_with(self);
    self.this_status = old_this_status;
    self.current_export_name = old_current_export_name;
    self.fn_decl_ident = old_fn_decl_ident;
  }

  fn visit_mut_arrow_expr(&mut self, a: &mut ArrowExpr) {
    // Arrow expressions need to be visited in prepass to determine if it's
    // an action function or not.
    let found_use_server =
      self.has_use_server_for_function(if let BlockStmtOrExpr::BlockStmt(block) = &mut *a.body {
        Some(block)
      } else {
        None
      });

    if found_use_server {
      self.this_status = ThisStatus::Forbidden;
    }

    let declared_idents_until = self.declared_idents.len();
    let old_names = take(&mut self.names);

    {
      // Visit children
      let old_in_module = replace(&mut self.in_module_level, false);
      let should_track_names = found_use_server || self.should_track_names;
      let old_should_track_names = replace(&mut self.should_track_names, should_track_names);
      let old_current_export_name = self.current_export_name.take();
      {
        for n in &mut a.params {
          collect_idents_in_pat(n, &mut self.declared_idents);
        }
      }
      a.visit_mut_children_with(self);
      self.in_module_level = old_in_module;
      self.should_track_names = old_should_track_names;
      self.current_export_name = old_current_export_name;
    }

    let mut child_names = take(&mut self.names);

    if self.should_track_names {
      self.names = [old_names, child_names.clone()].concat();
    }

    if found_use_server {
      let arrow_ident = self.arrow_or_fn_expr_ident.clone();

      if !self.validate_async_function(a.is_async, a.span, arrow_ident.as_ref()) {
        // If this is an exported arrow function that failed validation, remove it from
        // export_name_by_local_id so the post-pass doesn't register it.
        if self.current_export_name.is_some()
          && let Some(arrow_ident) = arrow_ident
        {
          self
            .export_name_by_local_id
            .swap_remove(&arrow_ident.to_id());
        }

        return;
      }

      // If this function is invalid, or any prior errors have been emitted, skip further
      // processing.
      if HANDLER.with(|handler| handler.has_errors()) {
        return;
      }

      // For server action files, register exports without hoisting (for both server and
      // client layers).
      if self.in_action_file
        && found_use_server
        && let Some(export_name) = self.current_export_name.clone()
      {
        let params: Vec<Param> = a.params.iter().map(|p| Param::from(p.clone())).collect();

        self.register_server_action_export(
          &export_name,
          arrow_ident.as_ref(),
          Some(&params),
          a.span,
          &mut || Box::new(Expr::Arrow(a.take())),
        );

        return;
      }

      // Collect all the identifiers defined inside the closure and used
      // in the action function. With deduplication.
      retain_names_from_declared_idents(
        &mut child_names,
        &self.declared_idents[..declared_idents_until],
      );

      self.rewrite_expr_to_proxy_expr =
        Some(self.maybe_hoist_and_create_proxy_for_server_action_arrow_expr(child_names, a));
    }
  }

  fn visit_mut_module(&mut self, m: &mut Module) {
    self.start_pos = m.span.lo;
    m.visit_mut_children_with(self);
  }

  fn visit_mut_stmt(&mut self, n: &mut Stmt) {
    n.visit_mut_children_with(self);

    if self.in_module_level {
      return;
    }

    // If it's a closure (not in the module level), we need to collect
    // identifiers defined in the closure.
    collect_decl_idents_in_stmt(n, &mut self.declared_idents);
  }

  fn visit_mut_param(&mut self, n: &mut Param) {
    n.visit_mut_children_with(self);

    if self.in_module_level {
      return;
    }

    collect_idents_in_pat(&n.pat, &mut self.declared_idents);
  }

  fn visit_mut_prop_or_spread(&mut self, n: &mut PropOrSpread) {
    let old_arrow_or_fn_expr_ident = self.arrow_or_fn_expr_ident.clone();
    let old_current_export_name = self.current_export_name.take();

    if let PropOrSpread::Prop(prop) = n {
      if let Prop::KeyValue(KeyValueProp {
        key: PropName::Ident(ident_name),
        value,
        ..
      }) = &**prop
      {
        if matches!(**value, Expr::Arrow(_) | Expr::Fn(_)) {
          self.current_export_name = None;
          self.arrow_or_fn_expr_ident = Some(ident_name.clone().into());
        }
      } else if let Prop::Method(MethodProp { key, .. }) = &**prop {
        let key = key.clone();

        if let PropName::Ident(ident_name) = &key {
          self.arrow_or_fn_expr_ident = Some(ident_name.clone().into());
        }

        let old_this_status = replace(&mut self.this_status, ThisStatus::Allowed);
        self.rewrite_expr_to_proxy_expr = None;
        self.current_export_name = None;
        n.visit_mut_children_with(self);
        self.current_export_name = old_current_export_name.clone();
        self.this_status = old_this_status;

        if let Some(expr) = self.rewrite_expr_to_proxy_expr.take() {
          *n = PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp { key, value: expr })));
        }

        return;
      }
    }

    if !self.in_module_level
      && self.should_track_names
      && let PropOrSpread::Prop(prop) = n
      && let Prop::Shorthand(i) = &**prop
    {
      self.names.push(Name::from(i));
      self.should_track_names = false;
      n.visit_mut_children_with(self);
      self.should_track_names = true;
      return;
    }

    n.visit_mut_children_with(self);
    self.arrow_or_fn_expr_ident = old_arrow_or_fn_expr_ident;
    self.current_export_name = old_current_export_name;
  }

  fn visit_mut_class(&mut self, n: &mut Class) {
    let old_this_status = replace(&mut self.this_status, ThisStatus::Allowed);
    n.visit_mut_children_with(self);
    self.this_status = old_this_status;
  }

  fn visit_mut_class_member(&mut self, n: &mut ClassMember) {
    if let ClassMember::Method(ClassMethod {
      is_abstract: false,
      is_static: true,
      kind: MethodKind::Method,
      key,
      span,
      accessibility: None | Some(Accessibility::Public),
      ..
    }) = n
    {
      let key = key.clone();
      let span = *span;
      let old_arrow_or_fn_expr_ident = self.arrow_or_fn_expr_ident.clone();

      if let PropName::Ident(ident_name) = &key {
        self.arrow_or_fn_expr_ident = Some(ident_name.clone().into());
      }

      let old_this_status = replace(&mut self.this_status, ThisStatus::Allowed);
      let old_current_export_name = self.current_export_name.take();
      self.rewrite_expr_to_proxy_expr = None;
      self.current_export_name = None;
      n.visit_mut_children_with(self);
      self.this_status = old_this_status;
      self.current_export_name = old_current_export_name;
      self.arrow_or_fn_expr_ident = old_arrow_or_fn_expr_ident;

      if let Some(expr) = self.rewrite_expr_to_proxy_expr.take() {
        *n = ClassMember::ClassProp(ClassProp {
          span,
          key,
          value: Some(expr),
          is_static: true,
          ..Default::default()
        });
      }
    } else {
      n.visit_mut_children_with(self);
    }
  }

  fn visit_mut_class_method(&mut self, n: &mut ClassMethod) {
    if n.is_static {
      n.visit_mut_children_with(self);
    } else if is_action_fn(&n.function.body) {
      emit_error(ServerActionsErrorKind::InlineUseServerInClassInstanceMethod { span: n.span });
    } else {
      n.visit_mut_children_with(self);
    }
  }

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if let Callee::Expr(expr) = &mut n.callee
      && let Expr::Ident(Ident { sym, .. }) = &**expr
      && (sym == "jsxDEV" || sym == "_jsxDEV")
    {
      // Do not visit the 6th arg in a generated jsxDEV call, which is a `this`
      // expression, to avoid emitting an error for using `this` if it's
      // inside of a server function. https://github.com/facebook/react/blob/9106107/packages/react/src/jsx/ReactJSXElement.js#L429
      if n.args.len() > 4 {
        for arg in &mut n.args[0..4] {
          arg.visit_mut_with(self);
        }
        return;
      }
    }

    let old_current_export_name = self.current_export_name.take();
    n.visit_mut_children_with(self);
    self.current_export_name = old_current_export_name;
  }

  fn visit_mut_callee(&mut self, n: &mut Callee) {
    let old_in_callee = replace(&mut self.in_callee, true);
    n.visit_mut_children_with(self);
    self.in_callee = old_in_callee;
  }

  fn visit_mut_expr(&mut self, n: &mut Expr) {
    if !self.in_module_level
      && self.should_track_names
      && let Ok(mut name) = Name::try_from(&*n)
    {
      if self.in_callee {
        // This is a callee i.e. `foo.bar()`,
        // we need to track the actual value instead of the method name.
        if !name.1.is_empty() {
          name.1.pop();
        }
      }

      self.names.push(name);
      self.should_track_names = false;
      n.visit_mut_children_with(self);
      self.should_track_names = true;
      return;
    }

    self.rewrite_expr_to_proxy_expr = None;
    n.visit_mut_children_with(self);
    if let Some(expr) = self.rewrite_expr_to_proxy_expr.take() {
      *n = *expr;
    }
  }

  fn visit_mut_module_items(&mut self, stmts: &mut Vec<ModuleItem>) {
    self.in_action_file = self.has_use_server_for_module(stmts);

    let in_action_file = self.in_action_file;

    let should_track_exports = in_action_file;

    // Pre-pass: Collect a mapping from local identifiers to export names for all exports
    // in server boundary files ('use server'). This mapping is used to:
    // 1. Set current_export_name when visiting exported functions/variables during the main
    //    pass.
    // 2. Register any remaining exports in the post-pass that weren't handled by the visitor.
    if should_track_exports {
      for stmt in stmts.iter() {
        match stmt {
          ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(export_default_expr)) => {
            if let Expr::Ident(ident) = &*export_default_expr.expr {
              self.export_name_by_local_id.insert(
                ident.to_id(),
                ModuleExportName::Ident(atom!("default").into()),
              );
            }
          }
          ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_default_decl)) => {
            // export default function foo() {}
            if let DefaultDecl::Fn(f) = &export_default_decl.decl
              && let Some(ident) = &f.ident
            {
              self.export_name_by_local_id.insert(
                ident.to_id(),
                ModuleExportName::Ident(atom!("default").into()),
              );
            }
          }
          ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) => {
            // export function foo() {} or export const bar = ...
            match &export_decl.decl {
              Decl::Fn(f) => {
                self
                  .export_name_by_local_id
                  .insert(f.ident.to_id(), ModuleExportName::Ident(f.ident.clone()));
              }
              Decl::Var(var) => {
                for decl in &var.decls {
                  // Collect all identifiers from the pattern and track which may
                  // need cache runtime wrappers. For destructuring patterns, we
                  // always need wrappers since we can't statically know if the
                  // destructured values are functions. For simple identifiers,
                  // check the init expression.
                  let mut idents = vec![];
                  collect_idents_in_pat(&decl.name, &mut idents);

                  let is_destructuring = !matches!(&decl.name, Pat::Ident(_));
                  let needs_wrapper = if is_destructuring {
                    true
                  } else if let Some(init) = &decl.init {
                    may_need_cache_runtime_wrapper(init)
                  } else {
                    false
                  };

                  for ident in idents {
                    self
                      .export_name_by_local_id
                      .insert(ident.to_id(), ModuleExportName::Ident(ident.clone()));

                    if needs_wrapper {
                      self
                        .local_ids_that_need_cache_runtime_wrapper_if_exported
                        .insert(ident.to_id());
                    }
                  }
                }
              }
              _ => {}
            }
          }
          ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named_export)) => {
            if named_export.src.is_none() {
              for spec in &named_export.specifiers {
                match spec {
                  ExportSpecifier::Named(ExportNamedSpecifier {
                    orig: ModuleExportName::Ident(orig),
                    exported: Some(exported),
                    is_type_only: false,
                    ..
                  }) => {
                    // export { foo as bar } or export { foo as "📙" }
                    self
                      .export_name_by_local_id
                      .insert(orig.to_id(), exported.clone());
                  }
                  ExportSpecifier::Named(ExportNamedSpecifier {
                    orig: ModuleExportName::Ident(orig),
                    exported: None,
                    is_type_only: false,
                    ..
                  }) => {
                    // export { foo }
                    self
                      .export_name_by_local_id
                      .insert(orig.to_id(), ModuleExportName::Ident(orig.clone()));
                  }
                  _ => {}
                }
              }
            }
          }
          ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl))) => {
            // Track which declarations need cache runtime wrappers if exported.
            for decl in &var_decl.decls {
              if let Pat::Ident(ident_pat) = &decl.name
                && let Some(init) = &decl.init
                && may_need_cache_runtime_wrapper(init)
              {
                self
                  .local_ids_that_need_cache_runtime_wrapper_if_exported
                  .insert(ident_pat.id.to_id());
              }
            }
          }
          ModuleItem::Stmt(Stmt::Decl(Decl::Fn(_fn_decl))) => {
            // Function declarations are known functions and don't need runtime
            // wrappers.
          }
          ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
            // Track all imports. We don't know if they're functions, so they need
            // runtime wrappers if they end up getting re-exported.
            for spec in &import_decl.specifiers {
              match spec {
                ImportSpecifier::Named(named) => {
                  self
                    .local_ids_that_need_cache_runtime_wrapper_if_exported
                    .insert(named.local.to_id());
                }
                ImportSpecifier::Default(default) => {
                  self
                    .local_ids_that_need_cache_runtime_wrapper_if_exported
                    .insert(default.local.to_id());
                }
                ImportSpecifier::Namespace(ns) => {
                  self
                    .local_ids_that_need_cache_runtime_wrapper_if_exported
                    .insert(ns.local.to_id());
                }
              }
            }
          }
          _ => {}
        }
      }
    }

    let old_annotations = self.annotations.take();
    let mut new = Vec::with_capacity(stmts.len());

    // Main pass: For each statement, validate exports in server boundary files,
    // visit and transform it, and add it to the output along with any hoisted items.
    for mut stmt in stmts.take() {
      if should_track_exports {
        let mut disallowed_export_span = DUMMY_SP;

        match &mut stmt {
          ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl { decl, span })) => match decl {
            Decl::Var(_)
            | Decl::Fn(_)
            | Decl::TsInterface(_)
            | Decl::TsTypeAlias(_)
            | Decl::TsEnum(_) => {}
            _ => {
              disallowed_export_span = *span;
            }
          },
          ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named)) => {
            if !named.type_only
              && let Some(_) = &named.src
            {
              // export { x } from './module'
              if named.specifiers.iter().any(|s| match s {
                ExportSpecifier::Namespace(_) | ExportSpecifier::Default(_) => true,
                ExportSpecifier::Named(s) => !s.is_type_only,
              }) {
                disallowed_export_span = named.span;
              }
            }
          }
          ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(ExportDefaultDecl {
            decl,
            span,
          })) => match decl {
            DefaultDecl::Fn(_) | DefaultDecl::TsInterfaceDecl(_) => {}
            _ => {
              disallowed_export_span = *span;
            }
          },
          ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(default_expr)) => {
            match &mut *default_expr.expr {
              Expr::Fn(_) | Expr::Arrow(_) | Expr::Ident(_) | Expr::Call(_) => {}
              _ => {
                disallowed_export_span = default_expr.span;
              }
            }
          }
          ModuleItem::ModuleDecl(ModuleDecl::ExportAll(ExportAll {
            span, type_only, ..
          })) => {
            if !*type_only {
              disallowed_export_span = *span;
            }
          }
          _ => {}
        }

        // Emit validation error if we found a disallowed export
        if disallowed_export_span != DUMMY_SP {
          emit_error(ServerActionsErrorKind::ExportedSyncFunction {
            span: disallowed_export_span,
          });
          return;
        }
      }

      stmt.visit_mut_with(self);

      let new_stmt = if let Some(expr) = self.rewrite_default_fn_expr_to_proxy_expr.take() {
        Some(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
          ExportDefaultExpr {
            span: DUMMY_SP,
            expr,
          },
        )))
      } else {
        Some(stmt)
      };

      if self.config.is_react_server_layer || !in_action_file {
        new.append(&mut self.hoisted_extra_items);
        if let Some(stmt) = new_stmt {
          new.push(stmt);
        }
        new.extend(self.annotations.drain(..).map(ModuleItem::Stmt));
        new.append(&mut self.extra_items);
      }
    }

    // Post-pass: For server boundary files, register any exports that weren't already
    // registered during the main pass.
    if should_track_exports {
      for (id, export_name) in &self.export_name_by_local_id {
        if self.reference_ids_by_export_name.contains_key(export_name) {
          continue;
        }

        self.server_reference_exports.push(ServerReferenceExport {
          ident: Ident::from(id.clone()),
          export_name: export_name.clone(),
          reference_id: self.generate_server_reference_id(export_name, None),
        });
      }
    }

    if in_action_file && !self.config.is_react_server_layer {
      self.reference_ids_by_export_name.extend(
        self
          .server_reference_exports
          .iter()
          .map(|e| (e.export_name.clone(), e.reference_id.clone())),
      );

      if !self.reference_ids_by_export_name.is_empty() {
        self.has_action |= in_action_file;
      }
    };

    // If it's compiled in the client layer, each export field needs to be
    // wrapped by a reference creation call.
    let create_ref_ident = private_ident!("createServerReference");

    let client_layer_import = (self.has_action && !self.config.is_react_server_layer).then(|| {
      // import { createServerReference } from 'react-server-dom-rspack/client'
      // createServerReference("action_id")
      ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
          span: DUMMY_SP,
          local: create_ref_ident.clone(),
          imported: None,
          is_type_only: false,
        })],
        src: Box::new(Str {
          span: DUMMY_SP,
          value: atom!("react-server-dom-rspack/client").into(),
          raw: None,
        }),
        type_only: false,
        with: None,
        phase: Default::default(),
      }))
    });

    let mut client_layer_exports = FxIndexMap::default();

    // If it's a "use server" file, all exports need to be annotated.
    if should_track_exports {
      let server_reference_exports = self.server_reference_exports.take();

      for ServerReferenceExport {
        ident,
        export_name,
        reference_id: ref_id,
      } in &server_reference_exports
      {
        if !self.config.is_react_server_layer {
          if matches!(export_name, ModuleExportName::Ident(i) if i.sym == *"default") {
            let export_expr =
              ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                span: DUMMY_SP,
                expr: Box::new(Expr::Call(CallExpr {
                  // In development, we generate these spans for sourcemapping
                  // with better logs/errors. For production, this is not
                  // generated because it would leak server code to the browser.
                  span: if self.config.is_react_server_layer || self.config.is_development {
                    self.comments.add_pure_comment(ident.span.lo);
                    ident.span
                  } else {
                    PURE_SP
                  },
                  callee: Callee::Expr(Box::new(Expr::Ident(create_ref_ident.clone()))),
                  args: vec![ref_id.clone().as_arg()],
                  ..Default::default()
                })),
              }));
            client_layer_exports.insert(
              atom!("default"),
              (
                vec![export_expr],
                ModuleExportName::Ident(atom!("default").into()),
                ref_id.clone(),
              ),
            );
          } else {
            let var_name = self.gen_action_ident();

            let var_ident = Ident::new(var_name.clone(), DUMMY_SP, self.private_ctxt);

            // Determine span for the variable name. In development, we generate these
            // spans for sourcemapping with better logs/errors. For production, this is
            // not generated because it would leak server code to the browser.
            let name_span = if self.config.is_react_server_layer || self.config.is_development {
              ident.span
            } else {
              DUMMY_SP
            };

            let var_decl = ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
              span: DUMMY_SP,
              kind: VarDeclKind::Const,
              decls: vec![VarDeclarator {
                span: DUMMY_SP,
                name: Pat::Ident(IdentName::new(var_name.clone(), name_span).into()),
                init: Some(Box::new(Expr::Call(CallExpr {
                  span: PURE_SP,
                  callee: Callee::Expr(Box::new(Expr::Ident(create_ref_ident.clone()))),
                  args: vec![ref_id.clone().as_arg()],
                  ..Default::default()
                }))),
                definite: false,
              }],
              ..Default::default()
            }))));

            // Determine the export name. In development, we generate these spans for
            // sourcemapping with better logs/errors. For production, this is not
            // generated because it would leak server code to the browser.
            let exported_name = if self.config.is_react_server_layer || self.config.is_development {
              export_name.clone()
            } else {
              strip_export_name_span(export_name)
            };

            let export_named = ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
              span: DUMMY_SP,
              specifiers: vec![ExportSpecifier::Named(ExportNamedSpecifier {
                span: DUMMY_SP,
                orig: ModuleExportName::Ident(var_ident),
                exported: Some(exported_name),
                is_type_only: false,
              })],
              src: None,
              type_only: false,
              with: None,
            }));

            client_layer_exports.insert(
              var_name,
              (
                vec![var_decl, export_named],
                export_name.clone(),
                ref_id.clone(),
              ),
            );
          }
        } else {
          self.annotations.push(Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(annotate_ident_as_server_reference(
              ident.clone(),
              ref_id.clone(),
              ident.span,
            )),
          }));
        }
      }

      // Ensure that the exports are functions by appending a runtime check:
      //
      //   import { ensureServerActions } from 'react-server-dom-rspack/server'
      //   ensureServerActions([action1, action2, ...])
      //
      // But it's only needed for the server layer, because on the client
      // layer they're transformed into references already.
      if self.has_action && self.config.is_react_server_layer {
        new.append(&mut self.extra_items);

        if !server_reference_exports.is_empty() {
          let ensure_ident = private_ident!("ensureServerActions");
          new.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
            span: DUMMY_SP,
            specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
              span: DUMMY_SP,
              local: ensure_ident.clone(),
              imported: None,
              is_type_only: false,
            })],
            src: Box::new(Str {
              span: DUMMY_SP,
              value: atom!("react-server-dom-rspack/server").into(),
              raw: None,
            }),
            type_only: false,
            with: None,
            phase: Default::default(),
          })));
          new.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(Expr::Call(CallExpr {
              span: DUMMY_SP,
              callee: Callee::Expr(Box::new(Expr::Ident(ensure_ident))),
              args: vec![ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Array(ArrayLit {
                  span: DUMMY_SP,
                  elems: server_reference_exports
                    .iter()
                    .map(|ServerReferenceExport { ident, .. }| {
                      Some(ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Ident(ident.clone())),
                      })
                    })
                    .collect(),
                })),
              }],
              ..Default::default()
            })),
          })));
        }

        // Append annotations to the end of the file.
        new.extend(self.annotations.drain(..).map(ModuleItem::Stmt));
      }
    }

    if self.has_action && self.config.is_react_server_layer {
      // Inlined actions are only allowed on the server layer.
      // import { registerServerReference } from 'react-server-dom-rspack/server'
      new.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
          span: DUMMY_SP,
          local: quote_ident!("registerServerReference").into(),
          imported: None,
          is_type_only: false,
        })],
        src: Box::new(Str {
          span: DUMMY_SP,
          value: atom!("react-server-dom-rspack/server").into(),
          raw: None,
        }),
        type_only: false,
        with: None,
        phase: Default::default(),
      })));

      let mut import_count = 1;

      // Encryption and decryption only happens when there are bound arguments.
      if self.has_server_reference_with_bound_args {
        // import { encryptActionBoundArgs, decryptActionBoundArgs } from
        // 'private-next-rsc-action-encryption'
        new.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
          span: DUMMY_SP,
          specifiers: vec![
            ImportSpecifier::Named(ImportNamedSpecifier {
              span: DUMMY_SP,
              local: quote_ident!("encryptActionBoundArgs").into(),
              imported: None,
              is_type_only: false,
            }),
            ImportSpecifier::Named(ImportNamedSpecifier {
              span: DUMMY_SP,
              local: quote_ident!("decryptActionBoundArgs").into(),
              imported: None,
              is_type_only: false,
            }),
          ],
          src: Box::new(Str {
            span: DUMMY_SP,
            value: atom!("private-next-rsc-action-encryption").into(),
            raw: None,
          }),
          type_only: false,
          with: None,
          phase: Default::default(),
        })));
        import_count += 1;
      }

      // Make them the first items
      new.rotate_right(import_count);
    }

    if self.has_action {
      // Flip the map and convert it to a FxIndexMap for deterministic
      // ordering in the server references comment.
      let export_names_ordered_by_reference_id = self
        .reference_ids_by_export_name
        .iter()
        .map(|(export_name, reference_id)| (reference_id, export_name))
        .collect::<FxIndexMap<_, _>>();

      if self.config.is_react_server_layer {
        self.set_action_ids(&export_names_ordered_by_reference_id);
      } else {
        self.set_action_ids(&export_names_ordered_by_reference_id);
        #[allow(clippy::unwrap_used)]
        new.push(client_layer_import.unwrap());
        new.rotate_right(1);
        new.extend(
          client_layer_exports
            .into_iter()
            .flat_map(|(_, (items, _, _))| items),
        );
      }
    }

    *stmts = new;

    self.annotations = old_annotations;
  }

  fn visit_mut_stmts(&mut self, stmts: &mut Vec<Stmt>) {
    let old_annotations = self.annotations.take();

    let mut new = Vec::with_capacity(stmts.len());
    for mut stmt in stmts.take() {
      stmt.visit_mut_with(self);

      new.push(stmt);
      new.append(&mut self.annotations);
    }

    *stmts = new;

    self.annotations = old_annotations;
  }

  fn visit_mut_jsx_attr(&mut self, attr: &mut JSXAttr) {
    let old_arrow_or_fn_expr_ident = self.arrow_or_fn_expr_ident.take();

    if let (Some(JSXAttrValue::JSXExprContainer(container)), JSXAttrName::Ident(ident_name)) =
      (&attr.value, &attr.name)
    {
      match &container.expr {
        JSXExpr::Expr(box Expr::Arrow(_)) | JSXExpr::Expr(box Expr::Fn(_)) => {
          self.arrow_or_fn_expr_ident = Some(ident_name.clone().into());
        }
        _ => {}
      }
    }

    attr.visit_mut_children_with(self);
    self.arrow_or_fn_expr_ident = old_arrow_or_fn_expr_ident;
  }

  fn visit_mut_var_declarator(&mut self, var_declarator: &mut VarDeclarator) {
    let old_current_export_name = self.current_export_name.take();
    let old_arrow_or_fn_expr_ident = self.arrow_or_fn_expr_ident.take();

    if let (Pat::Ident(ident), Some(box Expr::Arrow(_) | box Expr::Fn(_))) =
      (&var_declarator.name, &var_declarator.init)
    {
      if self.in_module_level
        && let Some(export_name) = self.export_name_by_local_id.get(&ident.to_id())
      {
        self.current_export_name = Some(export_name.clone());
      }

      self.arrow_or_fn_expr_ident = Some(ident.id.clone());
    }

    var_declarator.visit_mut_children_with(self);

    self.current_export_name = old_current_export_name;
    self.arrow_or_fn_expr_ident = old_arrow_or_fn_expr_ident;
  }

  fn visit_mut_assign_expr(&mut self, assign_expr: &mut AssignExpr) {
    let old_arrow_or_fn_expr_ident = self.arrow_or_fn_expr_ident.clone();

    if let (
      AssignTarget::Simple(SimpleAssignTarget::Ident(ident)),
      box Expr::Arrow(_) | box Expr::Fn(_),
    ) = (&assign_expr.left, &assign_expr.right)
    {
      self.arrow_or_fn_expr_ident = Some(ident.id.clone());
    }

    assign_expr.visit_mut_children_with(self);
    self.arrow_or_fn_expr_ident = old_arrow_or_fn_expr_ident;
  }

  fn visit_mut_this_expr(&mut self, n: &mut ThisExpr) {
    if matches!(self.this_status, ThisStatus::Forbidden) {
      emit_error(ServerActionsErrorKind::ForbiddenExpression {
        span: n.span,
        expr: "this".into(),
      });
    }
  }

  fn visit_mut_super(&mut self, n: &mut Super) {
    if matches!(self.this_status, ThisStatus::Forbidden) {
      emit_error(ServerActionsErrorKind::ForbiddenExpression {
        span: n.span,
        expr: "super".into(),
      });
    }
  }

  fn visit_mut_ident(&mut self, n: &mut Ident) {
    if n.sym == *"arguments" && matches!(self.this_status, ThisStatus::Forbidden) {
      emit_error(ServerActionsErrorKind::ForbiddenExpression {
        span: n.span,
        expr: "arguments".into(),
      });
    }
  }

  noop_visit_mut_type!();
}

fn retain_names_from_declared_idents(
  child_names: &mut Vec<Name>,
  current_declared_idents: &[Ident],
) {
  // Collect the names to retain in a separate vector
  let mut retained_names = Vec::new();

  for name in child_names.iter() {
    let mut should_retain = true;

    // Merge child_names. For example if both `foo.bar` and `foo.bar.baz` are used,
    // we only need to keep `foo.bar` as it covers the other.

    // Currently this is O(n^2) and we can potentially improve this to O(n log n)
    // by sorting or using a hashset.
    for another_name in child_names.iter() {
      if name != another_name && name.0 == another_name.0 && name.1.len() >= another_name.1.len() {
        let mut is_prefix = true;
        for i in 0..another_name.1.len() {
          if name.1[i] != another_name.1[i] {
            is_prefix = false;
            break;
          }
        }
        if is_prefix {
          should_retain = false;
          break;
        }
      }
    }

    if should_retain
      && current_declared_idents
        .iter()
        .any(|ident| ident.to_id() == name.0)
      && !retained_names.contains(name)
    {
      retained_names.push(name.clone());
    }
  }

  // Replace the original child_names with the retained names
  *child_names = retained_names;
}

/// Returns true if the expression may need a cache runtime wrapper.
/// Known functions and known non-functions return false.
fn may_need_cache_runtime_wrapper(expr: &Expr) -> bool {
  match expr {
    // Known functions - don't need wrapper
    Expr::Arrow(_) | Expr::Fn(_) => false,
    // Known non-functions - don't need wrapper
    Expr::Object(_) | Expr::Array(_) | Expr::Lit(_) => false,
    // Unknown/might be function - needs runtime check
    _ => true,
  }
}

fn assign_name_to_ident(ident: &Ident, name: &str) -> Stmt {
  Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Call(CallExpr {
      span: DUMMY_SP,
      callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(Ident::new(
          atom!("Object"),
          DUMMY_SP,
          SyntaxContext::empty(),
        ))),
        prop: MemberProp::Computed(ComputedPropName {
          span: DUMMY_SP,
          expr: Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: atom!("defineProperty").into(),
            raw: None,
          }))),
        }),
      }))),
      args: vec![
        // $action
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Ident(ident.clone())),
        },
        // "name"
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: atom!("name").into(),
            raw: None,
          }))),
        },
        // { value: $name }
        ExprOrSpread {
          spread: None,
          expr: Box::new(Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
              key: PropName::Ident(IdentName::new(atom!("value"), DUMMY_SP)),
              value: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: Wtf8Atom::from(name),
                raw: None,
              }))),
            })))],
          })),
        },
      ],
      type_args: None,
      ctxt: SyntaxContext::default(),
    })),
  })
}

fn annotate_ident_as_server_reference(ident: Ident, action_id: Atom, original_span: Span) -> Expr {
  // registerServerReference(reference, id, null)
  Expr::Call(CallExpr {
    span: original_span,
    callee: quote_ident!("registerServerReference").as_callee(),
    args: vec![
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Ident(ident)),
      },
      ExprOrSpread {
        spread: None,
        expr: Box::new(action_id.clone().into()),
      },
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
      },
    ],
    ..Default::default()
  })
}

fn bind_args_to_ident(ident: Ident, bound: Vec<Option<ExprOrSpread>>, action_id: Atom) -> Expr {
  // ident.bind(null, [encryptActionBoundArgs("id", arg1, arg2, ...)])
  Expr::Call(CallExpr {
    span: DUMMY_SP,
    callee: Expr::Member(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(ident.into()),
      prop: MemberProp::Ident(quote_ident!("bind")),
    })
    .as_callee(),
    args: vec![
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
      },
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Call(CallExpr {
          span: DUMMY_SP,
          callee: quote_ident!("encryptActionBoundArgs").as_callee(),
          args: std::iter::once(ExprOrSpread {
            spread: None,
            expr: Box::new(action_id.into()),
          })
          .chain(bound.into_iter().flatten())
          .collect(),
          ..Default::default()
        })),
      },
    ],
    ..Default::default()
  })
}

// Detects if two strings are similar (but not the same).
// This implementation is fast and simple as it allows only one
// edit (add, remove, edit, swap), instead of using a N^2 Levenshtein algorithm.
//
// Example of similar strings of "use server":
// "use servers",
// "use-server",
// "use sevrer",
// "use srever",
// "use servre",
// "user server",
//
// This avoids accidental typos as there's currently no other static analysis
// tool to help when these mistakes happen.
fn detect_similar_strings(a: &str, b: &str) -> bool {
  let mut a = a.chars().collect::<Vec<char>>();
  let mut b = b.chars().collect::<Vec<char>>();

  if a.len() < b.len() {
    (a, b) = (b, a);
  }

  if a.len() == b.len() {
    // Same length, get the number of character differences.
    let mut diff = 0;
    for i in 0..a.len() {
      if a[i] != b[i] {
        diff += 1;
        if diff > 2 {
          return false;
        }
      }
    }

    // Should be 1 or 2, but not 0.
    diff != 0
  } else {
    if a.len() - b.len() > 1 {
      return false;
    }

    // A has one more character than B.
    for i in 0..b.len() {
      if a[i] != b[i] {
        // This should be the only difference, a[i+1..] should be equal to b[i..].
        // Otherwise, they're not considered similar.
        // A: "use srerver"
        // B: "use server"
        //          ^
        return a[i + 1..] == b[i..];
      }
    }

    // This happens when the last character of A is an extra character.
    true
  }
}

// Check if the function or arrow function has action,
// without mutating the function body or erroring out.
// This is used to quickly determine if we need to use the module-level
// directives for this function or not.
fn is_action_fn(maybe_body: &Option<BlockStmt>) -> bool {
  let mut result = false;
  if let Some(body) = maybe_body {
    for stmt in body.stmts.iter() {
      match stmt {
        Stmt::Expr(ExprStmt {
          expr: box Expr::Lit(Lit::Str(Str { value, .. })),
          ..
        }) => {
          if value == "use server" {
            result = true;
            break;
          }
        }
        _ => break,
      }
    }
  }
  result
}

fn collect_idents_in_array_pat(elems: &[Option<Pat>], idents: &mut Vec<Ident>) {
  for elem in elems.iter().flatten() {
    match elem {
      Pat::Ident(ident) => {
        idents.push(ident.id.clone());
      }
      Pat::Array(array) => {
        collect_idents_in_array_pat(&array.elems, idents);
      }
      Pat::Object(object) => {
        collect_idents_in_object_pat(&object.props, idents);
      }
      Pat::Rest(rest) => {
        if let Pat::Ident(ident) = &*rest.arg {
          idents.push(ident.id.clone());
        }
      }
      Pat::Assign(AssignPat { left, .. }) => {
        collect_idents_in_pat(left, idents);
      }
      Pat::Expr(..) | Pat::Invalid(..) => {}
    }
  }
}

fn collect_idents_in_object_pat(props: &[ObjectPatProp], idents: &mut Vec<Ident>) {
  for prop in props {
    match prop {
      ObjectPatProp::KeyValue(KeyValuePatProp { value, .. }) => {
        // For { foo: bar }, only collect 'bar' (the local binding), not 'foo' (the property
        // key).
        match &**value {
          Pat::Ident(ident) => {
            idents.push(ident.id.clone());
          }
          Pat::Array(array) => {
            collect_idents_in_array_pat(&array.elems, idents);
          }
          Pat::Object(object) => {
            collect_idents_in_object_pat(&object.props, idents);
          }
          _ => {}
        }
      }
      ObjectPatProp::Assign(AssignPatProp { key, .. }) => {
        // For { foo }, 'foo' is both the property key and local binding.
        idents.push(key.id.clone());
      }
      ObjectPatProp::Rest(RestPat { arg, .. }) => {
        if let Pat::Ident(ident) = &**arg {
          idents.push(ident.id.clone());
        }
      }
    }
  }
}

fn collect_idents_in_var_decls(decls: &[VarDeclarator], idents: &mut Vec<Ident>) {
  for decl in decls {
    collect_idents_in_pat(&decl.name, idents);
  }
}

fn collect_idents_in_pat(pat: &Pat, idents: &mut Vec<Ident>) {
  match pat {
    Pat::Ident(ident) => {
      idents.push(ident.id.clone());
    }
    Pat::Array(array) => {
      collect_idents_in_array_pat(&array.elems, idents);
    }
    Pat::Object(object) => {
      collect_idents_in_object_pat(&object.props, idents);
    }
    Pat::Assign(AssignPat { left, .. }) => {
      collect_idents_in_pat(left, idents);
    }
    Pat::Rest(RestPat { arg, .. }) => {
      if let Pat::Ident(ident) = &**arg {
        idents.push(ident.id.clone());
      }
    }
    Pat::Expr(..) | Pat::Invalid(..) => {}
  }
}

fn collect_decl_idents_in_stmt(stmt: &Stmt, idents: &mut Vec<Ident>) {
  if let Stmt::Decl(decl) = stmt {
    match decl {
      Decl::Var(var) => {
        collect_idents_in_var_decls(&var.decls, idents);
      }
      Decl::Fn(fn_decl) => {
        idents.push(fn_decl.ident.clone());
      }
      _ => {}
    }
  }
}

struct DirectiveVisitor<'a> {
  config: &'a Config,
  location: DirectiveLocation,
  found_use_server: bool,
  in_action_file: bool,
  is_allowed_position: bool,
}

impl DirectiveVisitor<'_> {
  /**
   * Returns `true` if the statement contains a server directive.
   * The found directive is assigned to `DirectiveVisitor::directive`.
   */
  fn visit_stmt(&mut self, stmt: &Stmt) -> bool {
    let in_fn_body = matches!(self.location, DirectiveLocation::FunctionBody);
    let allow_inline = self.config.is_react_server_layer || self.in_action_file;

    match stmt {
      Stmt::Expr(ExprStmt {
        expr: box Expr::Lit(Lit::Str(Str { value, span, .. })),
        ..
      }) => {
        // Match `use server`
        if value == "use server" {
          if in_fn_body && !allow_inline {
            emit_error(ServerActionsErrorKind::InlineUseServerInClientComponent { span: *span })
          } else if self.found_use_server {
            emit_error(ServerActionsErrorKind::MultipleDirectives {
              span: *span,
              location: self.location.clone(),
            });
          } else if self.is_allowed_position {
            self.found_use_server = true;
            return true;
          } else {
            emit_error(ServerActionsErrorKind::MisplacedDirective {
              span: *span,
              directive: value.to_string_lossy().into_owned(),
              location: self.location.clone(),
            });
          }
        } else if detect_similar_strings(&value.to_string_lossy(), "use server") {
          // Detect typo of "use server"
          emit_error(ServerActionsErrorKind::MisspelledDirective {
            span: *span,
            directive: value.to_string_lossy().into_owned(),
            expected_directive: "use server".to_string(),
          });
        }
      }
      Stmt::Expr(ExprStmt {
        expr:
          box Expr::Paren(ParenExpr {
            expr: box Expr::Lit(Lit::Str(Str { value, .. })),
            ..
          }),
        span,
        ..
      }) => {
        // Match `("use server")`.
        if value == "use server" || detect_similar_strings(&value.to_string_lossy(), "use server") {
          if self.is_allowed_position {
            emit_error(ServerActionsErrorKind::WrappedDirective {
              span: *span,
              directive: "use server".to_string(),
            });
          } else {
            emit_error(ServerActionsErrorKind::MisplacedWrappedDirective {
              span: *span,
              directive: "use server".to_string(),
              location: self.location.clone(),
            });
          }
        }
      }
      _ => {
        // Directives must not be placed after other statements.
        self.is_allowed_position = false;
      }
    };

    false
  }
}

pub(crate) struct ClosureReplacer<'a> {
  used_ids: &'a [Name],
  private_ctxt: SyntaxContext,
}

impl ClosureReplacer<'_> {
  fn index(&self, e: &Expr) -> Option<usize> {
    let name = Name::try_from(e).ok()?;
    self.used_ids.iter().position(|used_id| *used_id == name)
  }
}

impl VisitMut for ClosureReplacer<'_> {
  fn visit_mut_expr(&mut self, e: &mut Expr) {
    e.visit_mut_children_with(self);

    if let Some(index) = self.index(e) {
      *e = Expr::Ident(Ident::new(
        // $$ACTION_ARG_0
        format!("$$ACTION_ARG_{index}").into(),
        DUMMY_SP,
        self.private_ctxt,
      ));
    }
  }

  fn visit_mut_prop_or_spread(&mut self, n: &mut PropOrSpread) {
    n.visit_mut_children_with(self);

    if let PropOrSpread::Prop(box Prop::Shorthand(i)) = n {
      let name = Name::from(&*i);
      if let Some(index) = self.used_ids.iter().position(|used_id| *used_id == name) {
        *n = PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
          key: PropName::Ident(i.clone().into()),
          value: Box::new(Expr::Ident(Ident::new(
            // $$ACTION_ARG_0
            format!("$$ACTION_ARG_{index}").into(),
            DUMMY_SP,
            self.private_ctxt,
          ))),
        })));
      }
    }
  }

  noop_visit_mut_type!();
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NamePart {
  prop: Atom,
  is_member: bool,
  optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Name(Id, Vec<NamePart>);

impl From<&'_ Ident> for Name {
  fn from(value: &Ident) -> Self {
    Name(value.to_id(), vec![])
  }
}

impl TryFrom<&'_ Expr> for Name {
  type Error = ();

  fn try_from(value: &Expr) -> Result<Self, Self::Error> {
    match value {
      Expr::Ident(i) => Ok(Name(i.to_id(), vec![])),
      Expr::Member(e) => e.try_into(),
      Expr::OptChain(e) => e.try_into(),
      _ => Err(()),
    }
  }
}

impl TryFrom<&'_ MemberExpr> for Name {
  type Error = ();

  fn try_from(value: &MemberExpr) -> Result<Self, Self::Error> {
    match &value.prop {
      MemberProp::Ident(prop) => {
        let mut obj: Name = value.obj.as_ref().try_into()?;
        obj.1.push(NamePart {
          prop: prop.sym.clone(),
          is_member: true,
          optional: false,
        });
        Ok(obj)
      }
      _ => Err(()),
    }
  }
}

impl TryFrom<&'_ OptChainExpr> for Name {
  type Error = ();

  fn try_from(value: &OptChainExpr) -> Result<Self, Self::Error> {
    match &*value.base {
      OptChainBase::Member(m) => match &m.prop {
        MemberProp::Ident(prop) => {
          let mut obj: Name = m.obj.as_ref().try_into()?;
          obj.1.push(NamePart {
            prop: prop.sym.clone(),
            is_member: false,
            optional: value.optional,
          });
          Ok(obj)
        }
        _ => Err(()),
      },
      OptChainBase::Call(_) => Err(()),
    }
  }
}

impl From<Name> for Box<Expr> {
  fn from(value: Name) -> Self {
    let mut expr = Box::new(Expr::Ident(value.0.into()));

    for NamePart {
      prop,
      is_member,
      optional,
    } in value.1.into_iter()
    {
      #[allow(clippy::replace_box)]
      if is_member {
        expr = Box::new(Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: expr,
          prop: MemberProp::Ident(IdentName::new(prop, DUMMY_SP)),
        }));
      } else {
        expr = Box::new(Expr::OptChain(OptChainExpr {
          span: DUMMY_SP,
          base: Box::new(OptChainBase::Member(MemberExpr {
            span: DUMMY_SP,
            obj: expr,
            prop: MemberProp::Ident(IdentName::new(prop, DUMMY_SP)),
          })),
          optional,
        }));
      }
    }

    expr
  }
}

fn emit_error(error_kind: ServerActionsErrorKind) {
  let (span, msg) = match error_kind {
    ServerActionsErrorKind::ExportedSyncFunction { span } => (
      span,
      formatdoc! {
        r#"
          Only async functions are allowed to be exported in a \"use server\" file.
        "#,
      },
    ),
    ServerActionsErrorKind::ForbiddenExpression { span, expr } => (
      span,
      formatdoc! {
          r#"
            Server Actions cannot use `{expr}`.
          "#,
      },
    ),
    ServerActionsErrorKind::InlineUseServerInClassInstanceMethod { span } => (
      span,
      formatdoc! {
        r#"
          It is not allowed to define inline "use server" annotated class instance methods.
          To define Server Actions, use functions, object method properties, or static class methods instead.
        "#
      },
    ),
    ServerActionsErrorKind::InlineUseServerInClientComponent { span } => (
      span,
      formatdoc! {
        r#"
          It is not allowed to define inline "use server" annotated Server Actions in Client Components.
          To use Server Actions in a Client Component, you can either export them from a separate file with "use server" at the top, or pass them down through props from a Server Component.
        "#
      },
    ),
    ServerActionsErrorKind::InlineSyncFunction { span } => (
      span,
      formatdoc! {
        r#"
          Server Actions must be async functions.
        "#,
      },
    ),
    ServerActionsErrorKind::MisplacedDirective {
      span,
      directive,
      location,
    } => (
      span,
      formatdoc! {
        r#"
          The "{directive}" directive must be at the top of the {location}.
        "#,
        location = match location {
          DirectiveLocation::Module => "file",
          DirectiveLocation::FunctionBody => "function body",
        }
      },
    ),
    ServerActionsErrorKind::MisplacedWrappedDirective {
      span,
      directive,
      location,
    } => (
      span,
      formatdoc! {
        r#"
          The "{directive}" directive must be at the top of the {location}, and cannot be wrapped in parentheses.
        "#,
        location = match location {
          DirectiveLocation::Module => "file",
          DirectiveLocation::FunctionBody => "function body",
        }
      },
    ),
    ServerActionsErrorKind::MisspelledDirective {
      span,
      directive,
      expected_directive,
    } => (
      span,
      formatdoc! {
        r#"
          Did you mean "{expected_directive}"? "{directive}" is not a supported directive name."
        "#
      },
    ),
    ServerActionsErrorKind::MultipleDirectives { span, location } => (
      span,
      formatdoc! {
        r#"
          Conflicting directives "use server" found in the same {location}. You cannot place both directives at the top of a {location}. Please remove one of them.
        "#,
        location = match location {
          DirectiveLocation::Module => "file",
          DirectiveLocation::FunctionBody => "function body",
        }
      },
    ),
    ServerActionsErrorKind::WrappedDirective { span, directive } => (
      span,
      formatdoc! {
        r#"
          The "{directive}" directive cannot be wrapped in parentheses.
        "#
      },
    ),
  };

  HANDLER.with(|handler| handler.struct_span_err(span, &msg).emit());
}

/// Strips span information from a ModuleExportName, replacing all spans with DUMMY_SP.
/// Used in production builds to prevent leaking source code information in source maps to browsers.
fn strip_export_name_span(export_name: &ModuleExportName) -> ModuleExportName {
  match export_name {
    ModuleExportName::Ident(i) => {
      ModuleExportName::Ident(Ident::new(i.sym.clone(), DUMMY_SP, i.ctxt))
    }
    ModuleExportName::Str(s) => ModuleExportName::Str(Str {
      span: DUMMY_SP,
      value: s.value.clone(),
      raw: None,
    }),
  }
}

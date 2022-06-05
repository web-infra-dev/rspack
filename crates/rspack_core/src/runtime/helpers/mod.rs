use std::sync::atomic::{AtomicBool, Ordering::SeqCst};

use anyhow::Result;
use once_cell::sync::Lazy;

use rspack_swc::{
  swc_common::{FileName, FilePathMapping, SourceMap, DUMMY_SP},
  swc_ecma_ast::{
    BlockStmt, CallExpr, Expr, ExprStmt, FnExpr, Function, Module, ModuleItem, ParenExpr, Stmt,
  },
  swc_ecma_parser::parse_file_as_module,
  swc_ecma_utils::{drop_span, prepend_stmts, ExprFactory},
};

use crate::Bundle;

fn parse(code: &str, name: &str) -> Vec<ModuleItem> {
  let cm = SourceMap::new(FilePathMapping::empty());
  let fm = cm.new_source_file(FileName::Custom(name.into()), code.into());
  parse_file_as_module(
    &fm,
    Default::default(),
    Default::default(),
    None,
    &mut vec![],
  )
  .map(|script| drop_span(script.body))
  .map_err(|_| {})
  .unwrap()
}

macro_rules! define {
  (
     $($name:ident, $func:ident)*
  ) => {
    $(
        pub fn $func(to: &mut Vec<ModuleItem>){
            static STMTS: Lazy<Vec<ModuleItem>> = Lazy::new(|| {
                parse(include_str!(concat!("_rs_", stringify!($name), ".js")), stringify!($name))
            });

            to.extend((*STMTS).clone())
        }
    )*
  };
}

#[derive(Default)]
pub struct RuntimeInjector {
  pub cjs_runtime_mark_as_esm: AtomicBool,
  // define_export: AtomicBool,
  // get_default_export: AtomicBool,
  // has_own_property: AtomicBool,
  pub cjs_runtime_browser: AtomicBool,
  pub cjs_runtime_node: AtomicBool,
  pub cjs_runtime_hmr: AtomicBool,
}

trait IntoIIFE {
  fn into_iife(self) -> CallExpr;
}

trait IntoStmts {
  fn into_stmts(self) -> Vec<Stmt>;
}

impl IntoStmts for Vec<ModuleItem> {
  fn into_stmts(self) -> Vec<Stmt> {
    self
      .into_iter()
      .filter_map(|stmt| stmt.stmt())
      .collect::<Vec<_>>()
  }
}

impl IntoIIFE for Vec<ModuleItem> {
  fn into_iife(self) -> CallExpr {
    let paren_expr = ParenExpr {
      span: DUMMY_SP,
      expr: Expr::Fn(FnExpr {
        ident: None,
        function: Function {
          params: Default::default(),
          decorators: Default::default(),
          span: DUMMY_SP,
          body: Some(BlockStmt {
            span: DUMMY_SP,
            stmts: self.into_stmts(),
          }),
          is_generator: false,
          is_async: false,
          type_params: None,
          return_type: None,
        },
      })
      .into(),
    };

    paren_expr.as_iife()
  }
}

macro_rules! impl_runtime_injector {
  (
     $($name:ident)*
  ) => {

    $(
        define!($name, $name);

        impl RuntimeInjector {
          paste::item! {
            pub fn [<add_ $name>] (to: &mut Vec<ModuleItem>) {
              let mut buf = vec![];
              $name(&mut buf);
              let call_expr = buf.into_iife();
              let module_item = ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(Expr::Call(call_expr)),
              }));

              to.push(module_item);
            }
        }
      }
    )*

  };
}

impl_runtime_injector!(cjs_runtime_bootstrap);
impl_runtime_injector!(cjs_runtime_dynamic_browser);
impl_runtime_injector!(cjs_runtime_dynamic_node);
impl_runtime_injector!(cjs_runtime_hmr);
impl_runtime_injector!(cjs_runtime_require);
impl_runtime_injector!(cjs_runtime_require_hot);
impl_runtime_injector!(cjs_runtime_mark_as_esm);

impl RuntimeInjector {
  pub fn new() -> Self {
    Self {
      cjs_runtime_mark_as_esm: false.into(),
      // define_export: false.into(),
      // get_default_export: false.into(),
      // has_own_property: false.into(),
      cjs_runtime_browser: false.into(),
      cjs_runtime_node: false.into(),
      cjs_runtime_hmr: false.into(),
    }
  }

  pub fn as_code(&self, bundle: &Bundle) -> Result<String> {
    let mut iifes = vec![];
    self.add_to(&mut iifes);

    let result = bundle.context.compiler.run(|| {
      bundle.context.compiler.print(
        &Module {
          body: iifes,
          span: DUMMY_SP,
          shebang: None,
        },
        None,
        None,
        false,
        Default::default(),
        Default::default(),
        &Default::default(),
        None,
        false,
        None,
        false,
        false,
      )
    })?;

    Ok(result.code)
  }

  pub fn add_to(&self, to: &mut Vec<ModuleItem>) {
    let mut helpers = vec![];

    Self::add_cjs_runtime_bootstrap(&mut helpers);

    if self.cjs_runtime_hmr.load(SeqCst) {
      Self::add_cjs_runtime_hmr(&mut helpers);
      Self::add_cjs_runtime_require_hot(&mut helpers);
    } else {
      Self::add_cjs_runtime_require(&mut helpers);
    }

    if self.cjs_runtime_browser.load(SeqCst) {
      Self::add_cjs_runtime_dynamic_browser(&mut helpers);
    }

    if self.cjs_runtime_node.load(SeqCst) {
      Self::add_cjs_runtime_dynamic_node(&mut helpers);
    }

    if self.cjs_runtime_mark_as_esm.load(SeqCst) {
      Self::add_cjs_runtime_mark_as_esm(&mut helpers);
    }

    prepend_stmts(to, helpers.into_iter())
  }
}

use std::sync::atomic::{AtomicBool, Ordering};

use once_cell::sync::Lazy;

use swc_common::DUMMY_SP;
use {
  swc_common::{FileName, FilePathMapping, Mark, SourceMap},
  swc_ecma_ast::{BlockStmt, CallExpr, Expr, FnExpr, Function, ModuleItem, ParenExpr, Stmt},
  swc_ecma_parser::parse_file_as_script,
  swc_ecma_utils::{drop_span, ExprFactory},
};

fn parse(code: &str, name: &str) -> Vec<Stmt> {
  let cm = SourceMap::new(FilePathMapping::empty());
  let fm = cm.new_source_file(FileName::Custom(name.into()), code.into());
  parse_file_as_script(
    &fm,
    Default::default(),
    Default::default(),
    None,
    &mut vec![],
  )
  .map(drop_span)
  .map(|module| module.body)
  .map_err(|e| unreachable!("Error occurred while parsing module: {:?}", e))
  .unwrap()
}

#[derive(Debug)]
struct HelperMark(Mark);

impl Default for HelperMark {
  fn default() -> Self {
    HelperMark(Mark::fresh(Mark::root()))
  }
}

#[derive(Debug, Default)]
pub struct Helpers {
  external: bool, // TODO: add external runtime, i.e. support `output.runtimeChunk === "single"` option
  mark: HelperMark,
  inner: Inner,
}

impl Helpers {
  pub fn new(external: bool) -> Self {
    Helpers {
      external,
      mark: Default::default(),
      inner: Default::default(),
    }
  }

  // you may identify a helper with this mark
  pub const fn mark(&self) -> Mark {
    self.mark.0
  }

  pub const fn external(&self) -> bool {
    self.external
  }
}

better_scoped_tls::scoped_tls! {
  // Runtime Helpers
  pub static HELPERS: Helpers
}

// temporarily use global helpers, but it should be moved to a per-bundler scope
// pub(crate) static HELPERS: Lazy<Helpers> = Lazy::new(|| Helpers::new(false));

macro_rules! add_to {
  ($buf:expr, $name:ident, $enabled: expr) => {{
    static STMTS: Lazy<Vec<Stmt>> = Lazy::new(|| {
      parse(
        include_str!(concat!("_cjs_runtime_", stringify!($name), ".js")),
        stringify!($name),
      )
    });

    if $enabled.load(Ordering::Relaxed) {
      $buf.extend((*STMTS).clone())
    }
  }};
}

pub(crate) struct InjectHelpers;

impl InjectHelpers {
  pub fn make_helpers_for_module(&self) -> Vec<ModuleItem> {
    // TODO: external helpers
    // let external = HELPERS.external();

    if self.is_helper_used() {
      self
        .build_helpers_iife()
        .into_iter()
        .map(ModuleItem::Stmt)
        .collect()
    } else {
      vec![]
    }
  }
}

#[macro_export]
macro_rules! define_helpers {
  (
      Helpers {
          $( $name:ident : ( $( $dep:ident ),* ), )*
      }
  ) => {
    #[derive(Default, Debug)]
    struct Inner {
      $( $name: AtomicBool, )*
    }

    impl Helpers {
      // mark helper as used
      $(
        pub fn $name(&self) {
          self.inner.$name.store(true, Ordering::Relaxed);

          if !self.external {
            $(
              self.$dep();
            )*
          }
        }
      )*
    }

    impl InjectHelpers {
      fn is_helper_used(&self) -> bool {
        let mut used = false;

        HELPERS.with(|helpers| {
          $(
              used |= helpers.inner.$name.load(Ordering::Relaxed);
          )*
        });

        used
      }

      #[allow(unused)]
      fn build_helpers(&self) -> Vec<Stmt> {
        let mut buf = vec![];

        HELPERS.with(|helpers| {
          $(
            add_to!(&mut buf, $name, helpers.inner.$name);
          )*
        });


        buf
      }

      fn build_helpers_iife(&self) -> Vec<Stmt> {
        use swc_ecma_ast::{ExprStmt, Stmt};
        let mut buf: Vec<Stmt> = vec![];

        HELPERS.with(|helpers| {
          $(
            let mut b = vec![];
            add_to!(&mut b, $name, helpers.inner.$name);
            if !b.is_empty() {
              buf.push(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(Expr::Call(b.into_iife())),
              }));
            }
          )*
        });

        buf
      }

    }

  };
}

// The order of this is quite important, since rustc expands macros in the order of appearance, which is the order of `build_helpers`
define_helpers!(Helpers {
  define: (),
  dynamic_browser: (define),
  dynamic_node: (define),
  hmr: (define),
  require_hot: (define, hmr),
  require: (define),
  jsonp: (define),
});

#[macro_export]
macro_rules! helper_expr {
  ($name:ident, $first_ident:ident . $($other: tt)+) => {{
    // enable helper

    $crate::HELPERS.with(|helpers| {
      helpers.$name();
      let mark = helpers.mark();

      let span = DUMMY_SP.apply_mark(mark);
      member_expr!(span, $first_ident.$($other)+)
    })
  }};

  ($name:ident, $token:tt) => {{
    // enable helper

    $crate::HELPERS.with(|helpers| {
      helpers.$name();
      let mark = helpers.mark();

      let span = DUMMY_SP.apply_mark(mark);
      quote_ident!(span, i)
    })
  }};
}

#[macro_export]
macro_rules! cjs_runtime_helper {
  ($name: ident, $first_ident: ident . $other: tt) => {{
    $crate::helper_expr!($name, $first_ident.$other).as_callee()
  }};

  ($name: ident, $token: tt) => {{
    $crate::helper_expr!($name, $token).as_callee()
  }};
}

trait IntoIIFE {
  fn into_iife(self) -> CallExpr;
}

impl IntoIIFE for Vec<Stmt> {
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
            stmts: self,
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

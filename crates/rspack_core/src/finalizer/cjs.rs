use rspack_swc::{
  swc_common,
  swc_ecma_ast::Module,
  swc_ecma_visit::{as_folder, Fold},
};

pub fn commonjs() -> impl Fold + 'static {
  CommonjsFolder::new()
}

struct CommonjsFolder {}

impl CommonjsFolder {
  pub fn new() -> Self {
    CommonjsFolder {}
  }
}

impl Fold for CommonjsFolder {
  fn fold_module(&mut self, mut n: Module) -> Module {
    let body = n.body;
    n.body = body;
    n
  }
}

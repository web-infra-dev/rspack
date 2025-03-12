use crate::{impl_module_methods, Module};

#[napi]
pub struct ContextModule {
  pub(crate) module: Module,
}

impl_module_methods!(ContextModule);

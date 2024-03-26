use std::sync::{Arc, RwLock};

/// rust support hooks
#[derive(PartialEq)]
pub enum Hook {
  ContextModuleFactoryAfterResolve,
  NormalModuleFactoryResolveForScheme,
  NormalModuleFactoryCreateModule,
}

impl From<String> for Hook {
  fn from(s: String) -> Self {
    match s.as_str() {
      "contextModuleFactoryAfterResolve" => Hook::ContextModuleFactoryAfterResolve,
      "normalModuleFactoryCreateModule" => Hook::NormalModuleFactoryCreateModule,
      "normalModuleFactoryResolveForScheme" => Hook::NormalModuleFactoryResolveForScheme,
      hook_name => panic!("{hook_name} is an invalid hook name"),
    }
  }
}

#[derive(Default, Clone)]
pub struct DisabledHooks(Arc<RwLock<Vec<Hook>>>);

impl DisabledHooks {
  pub fn set_disabled_hooks(&self, hooks: Vec<String>) {
    let mut disabled_hooks = self.0.write().expect("failed to write lock");
    *disabled_hooks = hooks.into_iter().map(Into::into).collect::<Vec<Hook>>();
  }

  pub fn is_hook_disabled(&self, hook: &Hook) -> bool {
    self.0.read().expect("").contains(hook)
  }
}

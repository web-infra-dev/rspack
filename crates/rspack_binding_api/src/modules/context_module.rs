use crate::{impl_module_methods, Module, MODULE_PROPERTIES_BUFFER};

#[napi]
#[repr(C)]
pub struct ContextModule {
  pub(crate) module: Module,
}

impl ContextModule {
  pub(crate) fn custom_into_instance(
    self,
    env: &napi::Env,
  ) -> napi::Result<napi::bindgen_prelude::ClassInstance<Self>> {
    MODULE_PROPERTIES_BUFFER.with(|ref_cell| {
      let mut properties = ref_cell.borrow_mut();
      properties.clear();

      Self::new_inherited(self, env, &mut *properties)
    })
  }
}

impl_module_methods!(ContextModule);

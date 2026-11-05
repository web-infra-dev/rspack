use napi::{Env, JsString, Result, bindgen_prelude::*};

use crate::module::ModuleHandle;

#[napi(ts_args_type = "module: Module")]
pub fn external_module_get_user_request<'env>(
  env: &'env Env,
  module: Object<'env>,
) -> Result<JsString<'env>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_ref(|_, module| {
    let module = module
      .as_external_module()
      .ok_or_else(|| Error::from_reason("Module is not an ExternalModule"))?;
    env.create_string(module.user_request())
  })
}

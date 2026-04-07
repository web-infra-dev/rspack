use napi::{
  Either,
  bindgen_prelude::{Object, Promise},
};
use napi_derive::napi;
use rspack_error::Error;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_module_replacement::{
  NormalModuleReplacementPluginOptions, NormalModuleReplacer,
};
use rspack_plugin_runtime_chunk::RuntimeChunkName;
use rustc_hash::FxHashMap;

use crate::{js_regex::JsRegExp, normal_module_factory::JsResolveData};

#[napi(object, object_to_js = false)]
pub struct RawNormalModuleReplacementPluginOptions {
  #[napi(ts_type = "RegExp")]
  pub resource_reg_exp: JsRegExp,
  #[napi(ts_type = "string | ((data: JsResolveData) => JsResolveData)")]
  pub new_resource: RawNormalModuleReplacer,
}

impl TryFrom<RawNormalModuleReplacementPluginOptions> for NormalModuleReplacementPluginOptions {
  type Error = rspack_error::Error;

  fn try_from(val: RawNormalModuleReplacementPluginOptions) -> Result<Self, Self::Error> {
    Ok(Self {
      resource_reg_exp: val.resource_reg_exp.try_into()?,
      new_resource: RawNormalModuleReplacerWrapper(val.new_resource).into(),
    })
  }
}

type RawNormalModuleReplacer = Either<String, ThreadsafeFunction<JsResolveData, JsResolveData>>;
struct RawNormalModuleReplacerWrapper(RawNormalModuleReplacer);

impl From<RawNormalModuleReplacerWrapper> for NormalModuleReplacer {
  fn from(value: RawNormalModuleReplacerWrapper) -> Self {
    match value.0 {
      Either::A(s) => Self::String(s),
      Either::B(f) => NormalModuleReplacer::Fn(Box::new(move |data, create_data| {
        let f = f.clone();
        Box::pin(async move {
          let new_data = f
            .call_with_sync(JsResolveData::from_nmf_data(data, create_data.as_deref()))
            .await?;
          new_data.update_nmf_data(data, create_data);
          Ok(())
        })
      })),
    }
  }
}

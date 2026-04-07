use napi::bindgen_prelude::FnArgs;
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_ignore::{CheckResourceContent, IgnorePluginOptions};

use crate::js_regex::JsRegExp;

type RawCheckResource = ThreadsafeFunction<FnArgs<(String, String)>, bool>;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawIgnorePluginOptions {
  #[napi(ts_type = "RegExp")]
  pub resource_reg_exp: Option<JsRegExp>,
  #[napi(ts_type = "RegExp")]
  pub context_reg_exp: Option<JsRegExp>,
  #[napi(ts_type = "(resource: string, context: string) => boolean")]
  pub check_resource: Option<RawCheckResource>,
}

impl TryFrom<RawIgnorePluginOptions> for IgnorePluginOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawIgnorePluginOptions) -> Result<Self, Self::Error> {
    Ok(Self {
      resource_reg_exp: value.resource_reg_exp.map(TryInto::try_into).transpose()?,
      context_reg_exp: value.context_reg_exp.map(TryInto::try_into).transpose()?,

      check_resource: value.check_resource.map(|check_resource| {
        CheckResourceContent::Fn(Box::new(move |resource, context| {
          let f = check_resource.clone();

          Box::pin(async move {
            f.call_with_sync((resource.to_owned(), context.to_owned()).into())
              .await
          })
        }))
      }),
    })
  }
}

use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_ignore::{CheckResourceContent, IgnorePluginOptions};
use rspack_regex::RspackRegex;

type RawCheckResource = ThreadsafeFunction<(String, String), bool>;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawIgnorePluginOptions {
  #[napi(ts_type = "RegExp")]
  pub resource_reg_exp: Option<RspackRegex>,
  #[napi(ts_type = "RegExp")]
  pub context_reg_exp: Option<RspackRegex>,
  #[napi(ts_type = "(resource: string, context: string) => boolean")]
  pub check_resource: Option<RawCheckResource>,
}

impl From<RawIgnorePluginOptions> for IgnorePluginOptions {
  fn from(value: RawIgnorePluginOptions) -> Self {
    Self {
      resource_reg_exp: value.resource_reg_exp,
      context_reg_exp: value.context_reg_exp,

      check_resource: value.check_resource.map(|check_resource| {
        CheckResourceContent::Fn(Box::new(move |resource, context| {
          let f = check_resource.clone();

          Box::pin(async move { f.call((resource.to_owned(), context.to_owned())).await })
        }))
      }),
    }
  }
}

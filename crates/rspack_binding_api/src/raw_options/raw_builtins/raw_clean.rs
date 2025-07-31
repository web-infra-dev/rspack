use futures::future::BoxFuture;
use napi_derive::napi;
use rspack_core::CleanOptions;
use rspack_napi::threadsafe_function::ThreadsafeFunction;

type RawKeepFunction = ThreadsafeFunction<String, bool>;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCleanPluginOptions {
  pub dry: bool,
  #[napi(ts_type = "(path: string) => boolean")]
  pub keep: Option<RawKeepFunction>,
}

impl From<RawCleanPluginOptions> for CleanOptions {
  fn from(value: RawCleanPluginOptions) -> Self {
    Self {
      dry: value.dry,
      keep: value.keep.map(|keep_fn| {
        let keep_fn: Box<dyn Fn(&str) -> BoxFuture<rspack_error::Result<bool>> + Send + Sync> =
          Box::new(move |path: &str| {
            let keep_fn = keep_fn.clone();
            let path = path.to_string();
            Box::pin(async move {
              keep_fn.call_with_sync(path).await.map_err(|e| {
                rspack_error::error!("CleanPlugin: failed to call keep function: {:?}", e)
              })
            })
          });
        keep_fn
      }),
    }
  }
}

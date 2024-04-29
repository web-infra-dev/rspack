use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_dynamic_entry::{DynamicEntryPluginOptions, EntryDynamicResult};

use crate::RawEntryOptions;

#[derive(Debug)]
#[napi(object)]
pub struct RawEntryDynamicResult {
  pub import: Vec<String>,
  pub options: RawEntryOptions,
}

pub type RawEntryDynamic = ThreadsafeFunction<(), Vec<RawEntryDynamicResult>>;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDynamicEntryPluginOptions {
  pub context: String,
  #[napi(ts_type = "() => Promise<Vec<RawEntryDynamicResult>>")]
  pub entry: RawEntryDynamic,
}

impl From<RawDynamicEntryPluginOptions> for DynamicEntryPluginOptions {
  fn from(opts: RawDynamicEntryPluginOptions) -> Self {
    Self {
      context: opts.context.into(),
      entry: Box::new(move || {
        let f = opts.entry.clone();
        Box::pin(async move {
          let raw_result = f.call(()).await?;
          let result = raw_result
            .into_iter()
            .map(
              |RawEntryDynamicResult { import, options }| EntryDynamicResult {
                import,
                options: options.into(),
              },
            )
            .collect::<Vec<_>>();
          Ok(result)
        })
      }),
    }
  }
}

use napi::Either;
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_runtime_chunk::{RuntimeChunkName, RuntimeChunkOptions};

#[napi(object, object_to_js = false)]
pub struct RawRuntimeChunkOptions {
  #[napi(ts_type = "string | ((entrypoint: { name: string }) => string)")]
  pub name: RawRuntimeChunkName,
}

impl From<RawRuntimeChunkOptions> for RuntimeChunkOptions {
  fn from(value: RawRuntimeChunkOptions) -> Self {
    Self {
      name: RawRuntimeChunkNameWrapper(value.name).into(),
    }
  }
}

type RawRuntimeChunkName = Either<String, ThreadsafeFunction<RawRuntimeChunkNameFnCtx, String>>;
struct RawRuntimeChunkNameWrapper(RawRuntimeChunkName);

#[napi(object)]
pub struct RawRuntimeChunkNameFnCtx {
  pub name: String,
}

impl From<RawRuntimeChunkNameWrapper> for RuntimeChunkName {
  fn from(value: RawRuntimeChunkNameWrapper) -> Self {
    match value.0 {
      Either::A(s) => {
        if s == "single" {
          Self::Single
        } else if s == "multiple" {
          Self::Multiple
        } else {
          Self::String(s)
        }
      }
      Either::B(f) => RuntimeChunkName::Fn(Box::new(move |name| {
        let f = f.clone();
        Box::pin(async move {
          f.call(RawRuntimeChunkNameFnCtx {
            name: name.to_string(),
          })
          .await
        })
      })),
    }
  }
}

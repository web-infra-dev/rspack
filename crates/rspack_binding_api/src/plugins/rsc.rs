use std::sync::Arc;

use futures::future::BoxFuture;
use napi::{
  Env, Status,
  bindgen_prelude::{
    External, ExternalRef, FromNapiValue, Function, JsObjectValue, Object, Promise, Reference,
    WeakReference,
  },
  threadsafe_function::ThreadsafeFunction,
};
use rspack_core::{Compiler, CompilerId};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_plugin_rsc::{Coordinator, RscClientPluginOptions};

use crate::JsCompiler;

type InvalidateTsFn = Arc<ThreadsafeFunction<(), (), (), Status, false, true, 0>>;

#[napi]
pub struct JsCoordinator {
  get_server_compiler_id_ts_fn:
    Arc<ThreadsafeFunction<(), &'static External<CompilerId>, (), Status, false, true, 0>>,
  inner: Option<Arc<Coordinator>>,
}

#[napi]
impl JsCoordinator {
  #[napi(constructor)]
  pub fn new(
    get_server_compiler_id_js_fn: Function<'static, (), &'static External<CompilerId>>,
  ) -> napi::Result<Self> {
    let get_server_compiler_id_ts_fn = Arc::new(
      get_server_compiler_id_js_fn
        .build_threadsafe_function::<()>()
        .callee_handled::<false>()
        .max_queue_size()
        .weak::<true>()
        .build()?,
    );

    Ok(Self {
      get_server_compiler_id_ts_fn,
      inner: None,
    })
  }
}

impl From<&mut JsCoordinator> for Arc<Coordinator> {
  fn from(value: &mut JsCoordinator) -> Self {
    if let Some(inner) = &value.inner {
      return inner.clone();
    }

    let get_server_compiler_id_ts_fn = value.get_server_compiler_id_ts_fn.clone();
    let get_server_compiler_id = Box::new(
      move || -> BoxFuture<'static, rspack_error::Result<CompilerId>> {
        let ts_fn = get_server_compiler_id_ts_fn.clone();
        Box::pin(async move {
          let external = ts_fn.call_async(()).await.to_rspack_result()?;
          Ok(**external)
        })
      },
    );

    let coordinator = Arc::new(Coordinator::new(get_server_compiler_id));
    value.inner = Some(coordinator.clone());
    coordinator
  }
}

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
  invalidate_server_compiler_ts_fn: InvalidateTsFn,
  invalidate_client_compiler_ts_fn: InvalidateTsFn,
  get_server_compiler_id_ts_fn:
    Arc<ThreadsafeFunction<(), &'static External<CompilerId>, (), Status, false, true, 0>>,
  coordinator: Option<Arc<Coordinator>>,
}

#[napi]
impl JsCoordinator {
  #[napi(constructor)]
  pub fn new(
    invalidate_server_compiler_js_fn: Function<'static, (), ()>,
    invalidate_client_compiler_js_fn: Function<'static, (), ()>,
    get_server_compiler_id_js_fn: Function<'static, (), &'static External<CompilerId>>,
  ) -> napi::Result<Self> {
    let invalidate_server_compiler_ts_fn = Arc::new(
      invalidate_server_compiler_js_fn
        .build_threadsafe_function::<()>()
        .callee_handled::<false>()
        .max_queue_size::<0>()
        .weak::<true>()
        .build()?,
    );

    let invalidate_client_compiler_ts_fn = Arc::new(
      invalidate_client_compiler_js_fn
        .build_threadsafe_function::<()>()
        .callee_handled::<false>()
        .max_queue_size::<0>()
        .weak::<true>()
        .build()?,
    );

    let get_server_compiler_id_ts_fn = Arc::new(
      get_server_compiler_id_js_fn
        .build_threadsafe_function::<()>()
        .callee_handled::<false>()
        .max_queue_size()
        .weak::<true>()
        .build()?,
    );

    Ok(Self {
      invalidate_server_compiler_ts_fn,
      invalidate_client_compiler_ts_fn,
      get_server_compiler_id_ts_fn,
      coordinator: None,
    })
  }
}

impl From<&mut JsCoordinator> for Arc<Coordinator> {
  fn from(value: &mut JsCoordinator) -> Self {
    if let Some(coordinator) = &value.coordinator {
      return coordinator.clone();
    }
    let invalidate_server_compiler_ts_fn = value.invalidate_server_compiler_ts_fn.clone();
    let invalidate_server_compiler =
      Box::new(move || -> BoxFuture<'static, rspack_error::Result<()>> {
        let ts_fn = invalidate_server_compiler_ts_fn.clone();
        Box::pin(async move { ts_fn.call_async(()).await.to_rspack_result() })
      });

    let invalidate_client_compiler_ts_fn = value.invalidate_client_compiler_ts_fn.clone();
    let invalidate_client_compiler =
      Box::new(move || -> BoxFuture<'static, rspack_error::Result<()>> {
        let ts_fn = invalidate_client_compiler_ts_fn.clone();
        Box::pin(async move { ts_fn.call_async(()).await.to_rspack_result() })
      });

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

    let result = Arc::new(Coordinator::new(
      invalidate_server_compiler,
      invalidate_client_compiler,
      get_server_compiler_id,
    ));
    value.coordinator = Some(result.clone());
    result
  }
}

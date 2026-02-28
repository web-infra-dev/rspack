use std::sync::Arc;

use futures::future::BoxFuture;
use napi::{
  Env, Status,
  bindgen_prelude::{
    ClassInstance, Either3, External, ExternalRef, FromNapiValue, Function, JsObjectValue, Null,
    Object, Promise, Reference, Undefined, WeakReference,
  },
  threadsafe_function::ThreadsafeFunction,
};
use once_cell::unsync::OnceCell;
use rspack_core::{Compiler, CompilerId};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_plugin_rsc::{Coordinator, RscClientPluginOptions, RscServerPluginOptions};

use crate::JsCompiler;

type InvalidateTsFn = Arc<ThreadsafeFunction<(), (), (), Status, false, true, 0>>;

#[napi]
pub struct JsCoordinator {
  i: Arc<Coordinator>,
}

#[napi]
impl JsCoordinator {
  #[napi(constructor)]
  pub fn new(
    get_server_compiler_id_js_fn: Function<'static, (), &'static External<CompilerId>>,
  ) -> napi::Result<Self> {
    let get_server_compiler_id = {
      let ts_fn = Arc::new(
        get_server_compiler_id_js_fn
          .build_threadsafe_function::<()>()
          .callee_handled::<false>()
          .max_queue_size::<0>()
          .weak::<true>()
          .build()?,
      );
      Box::new(
        move || -> BoxFuture<'static, rspack_error::Result<CompilerId>> {
          let ts_fn = ts_fn.clone();
          Box::pin(async move {
            let external = ts_fn.call_async(()).await.to_rspack_result()?;
            Ok(**external)
          })
        },
      )
    };

    Ok(Self {
      i: Arc::new(Coordinator::new(get_server_compiler_id)),
    })
  }
}

impl From<&JsCoordinator> for Arc<Coordinator> {
  fn from(value: &JsCoordinator) -> Self {
    value.i.clone()
  }
}

#[napi(object, object_to_js = false)]
pub struct JsRscClientPluginOptions<'a> {
  pub coordinator: ClassInstance<'a, JsCoordinator>,
}

impl From<&JsRscClientPluginOptions<'_>> for RscClientPluginOptions {
  fn from(value: &JsRscClientPluginOptions) -> Self {
    Self {
      coordinator: value.coordinator.i.clone(),
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct JsRscServerPluginOptions<'a> {
  pub coordinator: ClassInstance<'a, JsCoordinator>,
  pub on_server_component_changes: Option<Either3<Function<'static, (), ()>, Undefined, Null>>,
}

impl TryFrom<&JsRscServerPluginOptions<'_>> for RscServerPluginOptions {
  type Error = napi::Error;

  fn try_from(value: &JsRscServerPluginOptions) -> napi::Result<Self> {
    let on_server_component_changes: Option<
      Box<dyn Fn() -> BoxFuture<'static, rspack_error::Result<()>> + Sync + Send>,
    > = match value.on_server_component_changes {
      Some(Either3::A(js_fn)) => {
        let ts_fn = Arc::new(
          js_fn
            .build_threadsafe_function::<()>()
            .callee_handled::<false>()
            .max_queue_size::<0>()
            .weak::<true>()
            .build()?,
        );
        Some(Box::new(
          move || -> BoxFuture<'static, rspack_error::Result<()>> {
            let ts_fn = ts_fn.clone();
            Box::pin(async move { ts_fn.call_async(()).await.to_rspack_result() })
          },
        ))
      }
      _ => None,
    };

    Ok(Self {
      coordinator: value.coordinator.i.clone(),
      on_server_component_changes,
    })
  }
}

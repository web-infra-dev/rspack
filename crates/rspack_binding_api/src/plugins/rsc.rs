use std::sync::Arc;

use futures::future::BoxFuture;
use napi::{
  Env, Status,
  bindgen_prelude::{
    ClassInstance, Either, Either3, External, ExternalRef, FromNapiValue, Function, JsObjectValue,
    Null, Object, Promise, Reference, Undefined, WeakReference,
  },
  threadsafe_function::ThreadsafeFunction,
};
use once_cell::unsync::OnceCell;
use rspack_core::{Compiler, CompilerId};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_plugin_rsc::{
  Coordinator, OnManifest, RscClientPluginOptions, RscCssLinkProps, RscServerPluginOptions,
};

use crate::JsCompiler;

type InvalidateTsFn = Arc<ThreadsafeFunction<(), (), (), Status, false, true, 0>>;
type OnServerComponentChangesReturn = Either3<Promise<()>, Undefined, Null>;

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
pub struct JsRscCssLinkOptions<'a> {
  #[napi(ts_type = "string | false")]
  pub precedence: Option<Either<String, bool>>,
  #[napi(ts_type = "Record<string, string>")]
  pub props: Option<Object<'a>>,
}

#[napi(object, object_to_js = false)]
pub struct JsRscServerPluginOptions<'a> {
  pub coordinator: ClassInstance<'a, JsCoordinator>,
  pub css_link: Option<Either3<JsRscCssLinkOptions<'a>, Undefined, Null>>,
  #[napi(ts_type = "(() => void | Promise<void>) | undefined | null")]
  pub on_server_component_changes:
    Option<Either3<Function<'static, (), OnServerComponentChangesReturn>, Undefined, Null>>,
  pub on_manifest: Option<Either3<Function<'static, String, Promise<()>>, Undefined, Null>>,
}

fn object_to_css_link_props(object: Object<'_>) -> napi::Result<RscCssLinkProps> {
  let mut props = RscCssLinkProps::default();
  let keys = Object::keys(&object)?;
  for key in keys {
    if key == "precedence" {
      continue;
    }
    if let Some(value) = object.get::<String>(&key)? {
      props.insert(key, value);
    }
  }
  Ok(props)
}

fn normalize_css_link_props(
  css_link_props: Option<Either3<JsRscCssLinkOptions<'_>, Undefined, Null>>,
) -> napi::Result<RscCssLinkProps> {
  match css_link_props {
    None | Some(Either3::B(_) | Either3::C(_)) => {
      let mut props = RscCssLinkProps::default();
      props.insert("precedence".to_string(), "default".to_string());
      Ok(props)
    }
    Some(Either3::A(css_link_options)) => {
      let mut props = css_link_options
        .props
        .map(object_to_css_link_props)
        .transpose()?
        .unwrap_or_default();

      match css_link_options.precedence {
        Some(Either::A(precedence)) => {
          props.insert("precedence".to_string(), precedence);
        }
        Some(Either::B(false)) => {
          props.shift_remove("precedence");
        }
        Some(Either::B(true)) | None => {
          props.insert("precedence".to_string(), "default".to_string());
        }
      }

      Ok(props)
    }
  }
}

impl TryFrom<JsRscServerPluginOptions<'_>> for RscServerPluginOptions {
  type Error = napi::Error;

  fn try_from(value: JsRscServerPluginOptions) -> napi::Result<Self> {
    let on_server_component_changes: Option<
      Box<dyn Fn() -> BoxFuture<'static, rspack_error::Result<()>> + Sync + Send>,
    > = match &value.on_server_component_changes {
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
            Box::pin(async move {
              match ts_fn.call_async(()).await.to_rspack_result()? {
                Either3::A(promise) => promise.await.to_rspack_result(),
                Either3::B(_) | Either3::C(_) => Ok(()),
              }
            })
          },
        ))
      }
      _ => None,
    };

    let on_manifest: Option<OnManifest> = match &value.on_manifest {
      Some(Either3::A(js_fn)) => {
        let ts_fn = Arc::new(
          js_fn
            .build_threadsafe_function::<(String)>()
            .callee_handled::<false>()
            .max_queue_size::<0>()
            .weak::<true>()
            .build()?,
        );
        Some(Box::new(move |json: String| {
          let ts_fn = ts_fn.clone();
          Box::pin(async move {
            ts_fn
              .call_async(json)
              .await
              .to_rspack_result()?
              .await
              .to_rspack_result()
          })
        }))
      }
      _ => None,
    };

    Ok(Self {
      coordinator: value.coordinator.i.clone(),
      on_server_component_changes,
      on_manifest,
      css_link_props: normalize_css_link_props(value.css_link)?,
    })
  }
}

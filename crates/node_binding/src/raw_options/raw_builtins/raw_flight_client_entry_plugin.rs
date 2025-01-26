use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_next_flight_client_entry::{Options, ShouldInvalidateCbCtx, State};

#[napi(object, object_from_js = false)]
pub struct JsFlightClientEntryPluginState {}

impl From<State> for JsFlightClientEntryPluginState {
  fn from(val: State) -> Self {
    JsFlightClientEntryPluginState {}
  }
}

#[napi(object, object_from_js = false)]
pub struct JsShouldInvalidateCbCtx {
  pub entry_name: String,
  pub absolute_page_path: String,
  pub bundle_path: String,
  pub client_browser_loader: String,
}

impl From<ShouldInvalidateCbCtx> for JsShouldInvalidateCbCtx {
  fn from(val: ShouldInvalidateCbCtx) -> Self {
    JsShouldInvalidateCbCtx {
      entry_name: val.entry_name,
      absolute_page_path: val.absolute_page_path,
      bundle_path: val.bundle_path,
      client_browser_loader: val.client_browser_loader,
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct RawFlightClientEntryPluginOptions {
  pub dev: bool,
  pub app_dir: String,
  pub is_edge_server: bool,
  pub encryption_key: String,
  pub builtin_app_loader: bool,
  pub should_invalidate_cb: ThreadsafeFunction<JsShouldInvalidateCbCtx, bool>,
  pub state_cb: ThreadsafeFunction<JsFlightClientEntryPluginState, ()>,
}

impl From<RawFlightClientEntryPluginOptions> for Options {
  fn from(val: RawFlightClientEntryPluginOptions) -> Self {
    let should_invalidate_cb = val.should_invalidate_cb;
    let state_cb = val.state_cb;

    Options {
      dev: val.dev,
      app_dir: val.app_dir.into(),
      is_edge_server: val.is_edge_server,
      encryption_key: val.encryption_key,
      builtin_app_loader: val.builtin_app_loader,
      should_invalidate_cb: Box::new(move |ctx| {
        let js_ctx = ctx.into();
        let should_invalidate_cb = should_invalidate_cb.clone();
        should_invalidate_cb
          .blocking_call_with_sync(js_ctx)
          .unwrap()
      }),
      state_cb: Box::new(move |state| {
        let js_state = state.into();
        let state_cb = state_cb.clone();
        Box::pin(async move { state_cb.call(js_state).await })
      }),
    }
  }
}

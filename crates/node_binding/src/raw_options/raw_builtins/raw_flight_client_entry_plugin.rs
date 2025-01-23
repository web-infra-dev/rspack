use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_next_flight_client_entry::{Options, State};

#[napi(object, object_from_js = false)]
pub struct JsFlightClientEntryPluginState {}

impl From<State> for JsFlightClientEntryPluginState {
  fn from(val: State) -> Self {
    JsFlightClientEntryPluginState {}
  }
}

#[napi(object, object_to_js = false)]
pub struct RawFlightClientEntryPluginOptions {
  pub dev: bool,
  pub app_dir: String,
  pub is_edge_server: bool,
  pub encryption_key: String,
  pub state_cb: ThreadsafeFunction<JsFlightClientEntryPluginState, ()>,
}

impl From<RawFlightClientEntryPluginOptions> for Options {
  fn from(val: RawFlightClientEntryPluginOptions) -> Self {
    let state_cb = val.state_cb;
    Options {
      dev: val.dev,
      app_dir: val.app_dir.into(),
      is_edge_server: val.is_edge_server,
      encryption_key: val.encryption_key,
      state_cb: Box::new(move |state| {
        let js_state = state.into();
        let state_cb = state_cb.clone();
        Box::pin(async move { state_cb.call(js_state).await })
      }),
    }
  }
}

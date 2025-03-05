use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_next_flight_client_entry::{
  Action, ModuleInfo, ModulePair, Options, ShouldInvalidateCbCtx, State,
};
use rustc_hash::FxHashMap as HashMap;

#[napi(object, object_from_js = false)]
pub struct JsModuleInfo {
  pub module_id: String,
  pub is_async: bool,
}

impl From<ModuleInfo> for JsModuleInfo {
  fn from(value: ModuleInfo) -> Self {
    JsModuleInfo {
      module_id: value.module_id,
      is_async: value.is_async,
    }
  }
}

#[napi(object, object_from_js = false)]
pub struct JsAction {
  pub workers: HashMap<String, JsModuleInfo>,
  pub layer: HashMap<String, String>,
}

impl From<Action> for JsAction {
  fn from(action: Action) -> Self {
    let workers = action
      .workers
      .into_iter()
      .map(|(key, worker)| (key, worker.into()))
      .collect::<HashMap<String, JsModuleInfo>>();
    JsAction {
      workers,
      layer: action.layer,
    }
  }
}

fn into_js_actions(actions: HashMap<String, Action>) -> HashMap<String, JsAction> {
  actions
    .into_iter()
    .map(|(key, action)| (key, action.into()))
    .collect()
}

#[napi(object, object_from_js = false)]
pub struct JsModulePair {
  pub server: Option<JsModuleInfo>,
  pub client: Option<JsModuleInfo>,
}

impl From<ModulePair> for JsModulePair {
  fn from(module_pair: ModulePair) -> Self {
    JsModulePair {
      server: module_pair.server.map(Into::into),
      client: module_pair.client.map(Into::into),
    }
  }
}

fn into_js_module_pairs(
  module_pairs: HashMap<String, ModulePair>,
) -> HashMap<String, JsModulePair> {
  module_pairs
    .into_iter()
    .map(|(key, module_pair)| (key, module_pair.into()))
    .collect()
}

fn into_js_module_infos(
  module_infos: HashMap<String, ModuleInfo>,
) -> HashMap<String, JsModuleInfo> {
  module_infos
    .into_iter()
    .map(|(key, module_info)| (key, module_info.into()))
    .collect()
}

#[napi(object, object_from_js = false)]
pub struct JsFlightClientEntryPluginState {
  pub server_actions: HashMap<String, JsAction>,
  pub edge_server_actions: HashMap<String, JsAction>,

  pub server_action_modules: HashMap<String, JsModulePair>,
  pub edge_server_action_modules: HashMap<String, JsModulePair>,

  pub ssr_modules: HashMap<String, JsModuleInfo>,
  pub edge_ssr_modules: HashMap<String, JsModuleInfo>,

  pub rsc_modules: HashMap<String, JsModuleInfo>,
  pub edge_rsc_modules: HashMap<String, JsModuleInfo>,
  pub injected_client_entries: HashMap<String, String>,
}

impl From<State> for JsFlightClientEntryPluginState {
  fn from(state: State) -> Self {
    JsFlightClientEntryPluginState {
      server_actions: into_js_actions(state.server_actions),
      edge_server_actions: into_js_actions(state.edge_server_actions),

      server_action_modules: into_js_module_pairs(state.server_action_modules),
      edge_server_action_modules: into_js_module_pairs(state.edge_server_action_modules),

      ssr_modules: into_js_module_infos(state.ssr_modules),
      edge_ssr_modules: into_js_module_infos(state.edge_ssr_modules),

      rsc_modules: into_js_module_infos(state.rsc_modules),
      edge_rsc_modules: into_js_module_infos(state.edge_rsc_modules),
      injected_client_entries: state.injected_client_entries,
    }
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
  #[napi(ts_type = "(ctx: JsShouldInvalidateCbCtx) => boolean")]
  pub should_invalidate_cb: ThreadsafeFunction<JsShouldInvalidateCbCtx, bool>,
  #[napi(ts_type = "() => void")]
  pub invalidate_cb: ThreadsafeFunction<(), ()>,
  #[napi(ts_type = "(state: JsFlightClientEntryPluginState) => void")]
  pub state_cb: ThreadsafeFunction<JsFlightClientEntryPluginState, ()>,
}

impl From<RawFlightClientEntryPluginOptions> for Options {
  fn from(val: RawFlightClientEntryPluginOptions) -> Self {
    let should_invalidate_cb = val.should_invalidate_cb;
    let invalidate_cb = val.invalidate_cb;
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
      invalidate_cb: Box::new(move || {
        let invalidate_cb = invalidate_cb.clone();
        invalidate_cb.blocking_call_with_sync(()).unwrap()
      }),
      state_cb: Box::new(move |state| {
        let js_state = state.into();
        let state_cb = state_cb.clone();
        Box::pin(async move { state_cb.call(js_state).await })
      }),
    }
  }
}

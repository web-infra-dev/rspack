use std::cell::RefCell;

use napi::bindgen_prelude::BigInt;
use napi_derive::napi;
use rspack_tracing::{PerfettoTracer, StdoutTracer, TraceEvent, Tracer};
use rspack_util::tracing_preset::{
  TRACING_ALL_PRESET, TRACING_BENCH_TARGET, TRACING_OVERVIEW_PRESET,
};
use rustc_hash::FxHashMap as HashMap;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
  EnvFilter, Layer, Registry, layer::SubscriberExt, reload, util::SubscriberInitExt,
};

thread_local! {
  static GLOBAL_TRACE_STATE: RefCell<TraceState> = const { RefCell::new(TraceState::Uninitialized) };
}

#[napi(object)]
#[derive(Debug)]
pub struct RawTraceEvent {
  // event name
  pub name: String,
  // separate track name
  pub track_name: Option<String>,
  // separate group sliced name
  pub process_name: Option<String>,
  // extra debug arguments
  pub args: Option<HashMap<String, String>>,
  // track_uuid
  pub uuid: u32,
  // timestamp in microseconds
  pub ts: BigInt,
  // chrome trace event ph
  pub ph: String,
  // category
  pub categories: Option<Vec<String>>,
}

#[derive(Default)]
enum TraceState {
  /// uninitialized
  Uninitialized,
  /// initialized and turned on
  On(Box<dyn Tracer>, reload::Handle<EnvFilter, Registry>),
  /// initialized but turned off
  #[default]
  Off,
}

pub(super) fn register_global_trace(
  filter: String,
  layer: String,
  output: String,
) -> anyhow::Result<()> {
  let filter = match filter.as_str() {
    "OVERVIEW" => TRACING_OVERVIEW_PRESET,
    "ALL" => TRACING_ALL_PRESET,
    "BENCH" => TRACING_BENCH_TARGET,
    _ => filter.as_str(),
  };
  GLOBAL_TRACE_STATE.with(|state| {
    let mut state = state.borrow_mut();
    if let TraceState::Uninitialized = *state {
      let mut tracer: Box<dyn Tracer> = match layer.as_str() {
        "logger" => Box::new(StdoutTracer),
        "perfetto" => Box::new(PerfettoTracer::default()),
        _ => anyhow::bail!(
          "Unexpected layer: {}, supported layers:'logger', 'perfetto' ",
          layer
        ),
      };
      if let Some(layer) = tracer.setup(&output) {
        // SAFETY: we know that trace_var is `Ok(String)` now,
        // for the second unwrap, if we can't parse the directive, then the tracing result would be
        // unexpected, then panic is reasonable
        let (filter,reload_handle) = reload::Layer::new(EnvFilter::builder()
          .with_default_directive(LevelFilter::INFO.into())
          .with_regex(true)
          .parse(filter)
          .expect("Parse tracing directive syntax failed, for details about the directive syntax you could refer https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives")
      );
        tracing_subscriber::registry()
          .with(<_ as Layer<Registry>>::with_filter(layer, filter))
          .init();
        let new_state = TraceState::On(tracer, reload_handle);
        *state = new_state;
      };
    }
    Ok(())
  })
}

/// only the first call would take effect, the following calls would be ignored
pub(super) fn cleanup_global_trace() {
  GLOBAL_TRACE_STATE.with(|state| {
    let mut state = state.borrow_mut();
    match *state {
      TraceState::Uninitialized => {
        panic!("Global trace is not initialized, please call register_global_trace first");
      }
      TraceState::Off => {
        // do nothing, already cleaned up
      }
      TraceState::On(ref mut tracer, ref mut reload_handle) => {
        tracer.teardown();
        // turn off the tracing event
        let _ = reload_handle.modify(|filter| *filter = EnvFilter::new("off"));
        *state = TraceState::Off;
      }
    }
  });
}

/// sync Node.js event to Rust side
pub(super) fn sync_trace_event(events: Vec<RawTraceEvent>) {
  use std::borrow::BorrowMut;
  GLOBAL_TRACE_STATE.with(|state| {
    let mut state = state.borrow_mut();
    if let TraceState::On(tracer, _) = &mut **state.borrow_mut() {
      tracer.sync_trace(
        events
          .into_iter()
          .map(|event| TraceEvent {
            name: event.name,
            track_name: event.track_name,
            process_name: event.process_name,
            args: event
              .args
              .map(|args| args.into_iter().map(|(k, v)| (k, v.to_string())).collect()),
            uuid: event.uuid,
            ts: event.ts.get_u64().1,
            ph: event.ph,
            categories: event.categories,
          })
          .collect(),
      );
    }
  });
}

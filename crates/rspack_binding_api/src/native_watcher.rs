use std::{
  boxed::Box,
  path::{Path, PathBuf},
  sync::Arc,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use napi::bindgen_prelude::*;
use napi_derive::*;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_paths::InternedPath;
use rspack_regex::RspackRegex;
use rspack_watcher::{FsEventKind, FsWatcher, FsWatcherIgnored, FsWatcherOptions, IgnoredFn};

type JsWatcherIgnored = Either4<String, Vec<String>, RspackRegex, ThreadsafeFunction<String, bool>>;

fn to_fs_watcher_ignored(ignored: Option<JsWatcherIgnored>) -> FsWatcherIgnored {
  if let Some(ignored) = ignored {
    match ignored {
      Either4::A(path) => FsWatcherIgnored::Path(path),
      Either4::B(paths) => FsWatcherIgnored::Paths(paths),
      Either4::C(regex) => FsWatcherIgnored::Regex(regex),
      Either4::D(func) => FsWatcherIgnored::Function(to_ignored_fn(func)),
    }
  } else {
    FsWatcherIgnored::None
  }
}

fn to_ignored_fn(func: ThreadsafeFunction<String, bool>) -> IgnoredFn {
  Arc::new(move |path: String| {
    let func = func.clone();
    Box::pin(async move {
      match func.call_with_sync(path.clone()).await {
        Ok(ignored) => ignored,
        Err(e) => {
          tracing::error!("failed to call the `ignored` function with `{path}`: {e}");
          false
        }
      }
    })
  })
}

#[napi(object, object_to_js = false)]
pub struct NativeWatcherOptions {
  pub follow_symlinks: Option<bool>,

  pub poll_interval: Option<u32>,

  pub aggregate_timeout: Option<u32>,

  #[napi(ts_type = "string | string[] | RegExp | ((entry: string) => boolean)")]
  /// The ignored paths for the watcher.
  /// It can be a single path, an array of paths, a regular expression, or a
  /// predicate returning `true` for entries to ignore.
  pub ignored: Option<JsWatcherIgnored>,
}

#[napi]
pub struct NativeWatchResult {
  pub changed_files: Vec<String>,
  pub removed_files: Vec<String>,
}

/// A single, undelayed file system event delivered to the `callbackUndelayed`
/// callback. Passed as one object so napi-rs delivers it as a single JS
/// argument unambiguously (a tuple would arrive as an array).
#[napi(object)]
pub struct NativeWatchUndelayedEvent {
  pub kind: String,
  pub path: String,
}

#[napi]
pub struct NativeWatcher {
  watcher: FsWatcher,
  closed: bool,
  delivery: Option<Arc<JsDeliveryStream>>,
}

fn timestamp_to_system_time(millis: u64) -> SystemTime {
  UNIX_EPOCH + Duration::from_millis(millis)
}

#[napi]
impl NativeWatcher {
  #[napi(constructor)]
  pub fn new(options: NativeWatcherOptions) -> Self {
    let watcher = FsWatcher::new(
      FsWatcherOptions {
        follow_symlinks: options.follow_symlinks.unwrap_or(false),
        poll_interval: options.poll_interval,
        aggregate_timeout: options.aggregate_timeout,
      },
      to_fs_watcher_ignored(options.ignored),
    );

    Self {
      watcher,
      closed: false,
      delivery: None,
    }
  }

  #[napi]
  // Env is injected by N-API; the JavaScript watch signature is unchanged.
  #[allow(clippy::too_many_arguments)]
  pub fn watch(
    &mut self,
    env: Env,
    files: (Vec<String>, Vec<String>),
    directories: (Vec<String>, Vec<String>),
    missing: (Vec<String>, Vec<String>),
    start_time: BigInt,
    #[napi(ts_arg_type = "(err: Error | null, result: NativeWatchResult) => void")]
    callback: Function<'static, AggregateArgs, ()>,
    #[napi(ts_arg_type = "(event: NativeWatchUndelayedEvent) => void")]
    callback_undelayed: Function<'static, NativeWatchUndelayedEvent, ()>,
  ) -> napi::Result<()> {
    if self.closed {
      return Err(napi::Error::from_reason(
        "The native watcher has been closed, cannot watch again.",
      ));
    }

    let delivery = match &self.delivery {
      Some(delivery) => Arc::clone(delivery),
      None => {
        let delivery = Arc::new(JsDeliveryStream::new(&env)?);
        self.delivery = Some(Arc::clone(&delivery));
        delivery
      }
    };
    let callbacks = Arc::new(JsCallbacks {
      aggregate: callback.create_ref()?,
      undelayed: callback_undelayed.create_ref()?,
    });
    let js_event_handler = JsEventHandler {
      delivery: Arc::clone(&delivery),
      callbacks: Arc::clone(&callbacks),
    };
    let js_event_handler_undelayed = JsEventHandler {
      delivery,
      callbacks,
    };

    let start_time = start_time.get_u64().1;

    // `FsWatcher::watch` has already enqueued the request by the time it
    // returns; the future only signals "applied", so dropping it cancels
    // nothing.
    #[allow(clippy::let_underscore_future)]
    let _ = self.watcher.watch(
      to_tuple_path_iterator(files),
      to_tuple_path_iterator(directories),
      to_tuple_path_iterator(missing),
      timestamp_to_system_time(start_time),
      Box::new(js_event_handler),
      Box::new(js_event_handler_undelayed),
    );

    Ok(())
  }

  #[napi(ts_type = "(kind: 'change' | 'remove' | 'create', path: string): void")]
  pub fn trigger_event(&self, kind: String, path: String) {
    if let Some(kind) = match kind.as_str() {
      "change" => Some(FsEventKind::Change),
      "remove" => Some(FsEventKind::Remove),
      "create" => Some(FsEventKind::Create),
      _ => None,
    } {
      self
        .watcher
        .trigger_event(&InternedPath::from(AsRef::<Path>::as_ref(&path)), kind);
    }
  }

  #[napi(ts_return_type = "Promise<void>")]
  pub fn close<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    self.closed = true;
    // Handler references keep the stream alive until the ordered close aborts
    // both executor tasks. Already queued messages retain their own callbacks.
    self.delivery.take();

    // Call outside the async block: the synchronous enqueue keeps close
    // ordered behind preceding `watch` calls.
    let closing = self.watcher.close();

    rspack_napi::runtime::promise_from_future(env, async move {
      closing
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))
    })
  }

  #[napi]
  pub fn pause(&self) -> napi::Result<()> {
    self
      .watcher
      .pause()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(())
  }
}

fn to_tuple_path_iterator(
  tuple: (Vec<String>, Vec<String>),
) -> (
  impl Iterator<Item = InternedPath>,
  impl Iterator<Item = InternedPath>,
) {
  (
    tuple
      .0
      .into_iter()
      .map(|s| InternedPath::from(PathBuf::from(s))),
    tuple
      .1
      .into_iter()
      .map(|s| InternedPath::from(PathBuf::from(s))),
  )
}

// A single stream survives watch() callback replacement. Each queued message
// retains the callbacks of the generation that produced it.
type AggregateArgs = FnArgs<(
  Either<Null, napi::Error>,
  Either<NativeWatchResult, Undefined>,
)>;
type DeliveryFunction = napi::threadsafe_function::ThreadsafeFunction<
  JsDelivery,
  (),
  JsDispatchArgs,
  Status,
  false,
  false,
  0,
>;

struct JsCallbacks {
  aggregate: FunctionRef<AggregateArgs, ()>,
  undelayed: FunctionRef<NativeWatchUndelayedEvent, ()>,
}

enum JsDelivery {
  Aggregate(Arc<JsCallbacks>, napi::Result<NativeWatchResult>),
  Undelayed(Arc<JsCallbacks>, NativeWatchUndelayedEvent),
}

// Convert only the callback and its arguments; never invoke user code here.
struct JsDispatchArgs(JsDelivery);

impl JsValuesTupleIntoVec for JsDispatchArgs {
  fn into_vec(self, raw_env: napi::sys::napi_env) -> napi::Result<Vec<napi::sys::napi_value>> {
    let env = Env::from_raw(raw_env);
    match self.0 {
      JsDelivery::Aggregate(callbacks, result) => {
        let callback = callbacks.aggregate.borrow_back(&env)?;
        match result {
          Ok(result) => FnArgs::from((callback, true, Null, result)).into_vec(raw_env),
          Err(error) => FnArgs::from((callback, true, error, ())).into_vec(raw_env),
        }
      }
      JsDelivery::Undelayed(callbacks, event) => {
        let callback = callbacks.undelayed.borrow_back(&env)?;
        FnArgs::from((callback, false, Null, event)).into_vec(raw_env)
      }
    }
  }
}

struct JsDeliveryStream {
  inner: DeliveryFunction,
}

impl JsDeliveryStream {
  fn new(env: &Env) -> napi::Result<Self> {
    // A fixed, trusted expression: no user source, module lookup, or global
    // mutation. Keep user invocation in JS so *any* thrown value follows the
    // TSFN's original pending-exception path without Rust Error conversion.
    let dispatch: Function<'_, (), ()> = env.run_script(
      "(callback, aggregate, error, value) => aggregate ? callback(error, value) : callback(value)",
    )?;
    let inner = dispatch
      .build_threadsafe_function::<JsDelivery>()
      .callee_handled::<false>()
      .weak::<false>()
      // Nonblocking with no queue bound: a blocked JS thread cannot cause
      // QueueFull to silently discard a raw event or an aggregate snapshot.
      .max_queue_size::<0>()
      .build_callback(
        |ctx: napi::threadsafe_function::ThreadSafeCallContext<JsDelivery>| {
          Ok(JsDispatchArgs(ctx.value))
        },
      )?;
    Ok(Self { inner })
  }

  fn send(&self, event: JsDelivery) -> rspack_error::Result<()> {
    let status = self.inner.call(
      event,
      napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
    );
    match status {
      Status::Ok => Ok(()),
      // JS environment teardown cannot accept more callbacks. Return an error
      // to stop raw delivery for this batch; never retry against a closing env.
      Status::Closing => Err(rspack_error::error!(
        "Native watcher JavaScript delivery is closing"
      )),
      other => Err(rspack_error::error!(
        "Native watcher JavaScript delivery failed: {other:?}"
      )),
    }
  }
}

struct JsEventHandler {
  delivery: Arc<JsDeliveryStream>,
  callbacks: Arc<JsCallbacks>,
}

impl rspack_watcher::EventAggregateHandler for JsEventHandler {
  fn on_event_handle(
    &self,
    changed_files: rspack_util::fx_hash::FxHashSet<String>,
    deleted_files: rspack_util::fx_hash::FxHashSet<String>,
  ) {
    let result = NativeWatchResult {
      changed_files: changed_files.into_iter().collect(),
      removed_files: deleted_files.into_iter().collect(),
    };
    if let Err(error) = self.delivery.send(JsDelivery::Aggregate(
      Arc::clone(&self.callbacks),
      Ok(result),
    )) {
      tracing::debug!("{error}");
    }
  }

  fn on_error(&self, error: rspack_error::Error) {
    if let Err(error) = self.delivery.send(JsDelivery::Aggregate(
      Arc::clone(&self.callbacks),
      Err(napi::Error::from_reason(format!("Watcher error: {error}"))),
    )) {
      tracing::debug!("{error}");
    }
  }
}

impl rspack_watcher::EventHandler for JsEventHandler {
  fn on_change(&self, changed_file: String) -> rspack_error::Result<()> {
    self.delivery.send(JsDelivery::Undelayed(
      Arc::clone(&self.callbacks),
      NativeWatchUndelayedEvent {
        kind: "change".to_string(),
        path: changed_file,
      },
    ))
  }

  fn on_delete(&self, deleted_file: String) -> rspack_error::Result<()> {
    self.delivery.send(JsDelivery::Undelayed(
      Arc::clone(&self.callbacks),
      NativeWatchUndelayedEvent {
        kind: "remove".to_string(),
        path: deleted_file,
      },
    ))
  }
}

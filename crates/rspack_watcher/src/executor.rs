use std::{
  sync::{Arc, Mutex, MutexGuard, Weak},
  time::Duration,
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet as HashSet;
use tokio::{
  sync::mpsc,
  task::JoinHandle,
  time::{Instant, sleep_until},
};

use crate::{
  EventAggregateHandler, EventHandler, FsEvent, FsEventKind, ResolvedEvents,
  paths::PathManager,
  paused::{BufferedEvent, PausedEvents},
  trigger::Input,
};

#[derive(Default, Clone)]
pub(crate) struct FilesData {
  pub(crate) changed: HashSet<String>,
  pub(crate) deleted: HashSet<String>,
}

impl FilesData {
  pub(super) fn record(&mut self, events: &[FsEvent]) {
    for event in events {
      let path = event.path.to_string_lossy();
      if event.kind == FsEventKind::Remove {
        self.changed.remove(path.as_ref());
        self.deleted.insert(path.into_owned());
      } else {
        self.deleted.remove(path.as_ref());
        self.changed.insert(path.into_owned());
      }
    }
  }

  fn is_empty(&self) -> bool {
    self.changed.is_empty() && self.deleted.is_empty()
  }
}

#[derive(Default)]
struct DeliveryState {
  files: FilesData,
  paused: bool,
  pending: PausedEvents,
  pause_revision: u64,
  watch_revision: u64,
  handler: Option<mpsc::UnboundedSender<Message>>,
  deadline: Option<Instant>,
}

enum Message {
  Events(ResolvedEvents),
  Resume,
}

#[derive(Clone, Copy)]
pub(crate) struct WatchCycle {
  watch_revision: u64,
  pause_revision: u64,
}

pub(crate) struct Delivery {
  state: Mutex<DeliveryState>,
  timeout: Duration,
  #[cfg(feature = "testing")]
  pub(crate) trace: crate::testing::ExecutorTrace,
}

impl Delivery {
  fn lock(&self) -> MutexGuard<'_, DeliveryState> {
    self.state.lock().expect("watch deliveries poisoned")
  }

  pub(crate) fn new(timeout: Option<u32>) -> Self {
    Self {
      state: Mutex::new(DeliveryState {
        paused: true,
        ..Default::default()
      }),
      timeout: Duration::from_millis(u64::from(timeout.unwrap_or(50))),
      #[cfg(feature = "testing")]
      trace: Default::default(),
    }
  }

  pub(crate) fn pause(&self) {
    let mut state = self.lock();
    state.pause_revision += 1;
    state.paused = true;
    state.deadline = None;
  }

  pub(crate) fn pause_revision(&self) -> u64 {
    self.lock().pause_revision
  }

  pub(crate) fn begin_watch(&self, pause_revision: u64) -> WatchCycle {
    let mut state = self.lock();
    state.paused = true;
    state.deadline = None;
    state.watch_revision += 1;
    WatchCycle {
      watch_revision: state.watch_revision,
      pause_revision,
    }
  }

  pub(crate) fn finish_watch(&self, cycle: WatchCycle, paths: &PathManager) -> bool {
    let mut state = self.lock();
    if state.watch_revision != cycle.watch_revision || state.pause_revision != cycle.pause_revision
    {
      return false;
    }
    state
      .files
      .changed
      .retain(|p| paths.is_registered(&p.as_str().into()));
    state
      .files
      .deleted
      .retain(|p| paths.is_registered(&p.as_str().into()));
    if !state.files.is_empty() {
      state.deadline = Some(Instant::now() + self.timeout);
      if let Some(tx) = &state.handler {
        let _ = tx.send(Message::Resume);
      }
    }
    state.pending.retain_subscriptions(paths);
    state.paused = false;
    true
  }

  pub(crate) fn aggregated(&self, take: bool) -> FilesData {
    let mut state = self.lock();
    let mut files = if take {
      std::mem::take(&mut state.files)
    } else {
      state.files.clone()
    };
    state.pending.project(&mut files, take);
    files
  }

  pub(crate) fn publish(
    &self,
    input: Option<Input>,
    events: Option<ResolvedEvents>,
    subscribers: Vec<Weak<InternedPath>>,
  ) {
    let mut state = self.lock();
    if state.paused {
      state.pending.push(input, events, subscribers);
      return;
    }
    self.dispatch(&mut state, events);
  }

  pub(crate) fn next_replay(&self, cycle: WatchCycle) -> Option<BufferedEvent> {
    let state = self.lock();
    if state.paused || state.watch_revision != cycle.watch_revision {
      return None;
    }
    state.pending.front()
  }

  pub(crate) fn complete_replay(&self, cycle: WatchCycle, events: Option<ResolvedEvents>) {
    let mut state = self.lock();
    if !state.paused && state.watch_revision == cycle.watch_revision {
      let events = state.pending.finish(events);
      self.dispatch(&mut state, events);
    } else {
      state.pending.interrupted(events);
    }
  }

  fn take_due_batch(
    &self,
    deadline: Instant,
    rx: &mpsc::UnboundedReceiver<Message>,
  ) -> Option<FilesData> {
    let mut state = self.lock();
    if state.deadline != Some(deadline) || !rx.is_empty() {
      return None;
    }
    state.deadline = None;
    let files = std::mem::take(&mut state.files);
    if files.is_empty() {
      return None;
    }
    #[cfg(feature = "testing")]
    self.trace.lock().expect("executor trace poisoned").push(
      crate::testing::ExecutorEvent::Aggregated {
        at: Instant::now().into_std(),
        changed_files: files.changed.clone(),
        deleted_files: files.deleted.clone(),
      },
    );
    Some(files)
  }

  fn dispatch(&self, state: &mut DeliveryState, events: Option<ResolvedEvents>) {
    let Some(events) = events.filter(|events| !events.dependencies.is_empty()) else {
      return;
    };
    let Some(sender) = state.handler.as_ref() else {
      return;
    };
    let at = Instant::now();
    state.files.record(&events.dependencies);
    state.deadline = Some(at + self.timeout);
    #[cfg(feature = "testing")]
    self.trace.lock().expect("executor trace poisoned").push(
      crate::testing::ExecutorEvent::Accepted {
        at: at.into_std(),
        events: events
          .dependencies
          .iter()
          .map(|e| (e.path.to_string_lossy().into_owned(), e.kind))
          .collect(),
      },
    );
    let _ = sender.send(Message::Events(events));
  }
}

pub(crate) struct Executor {
  pub(crate) delivery: Arc<Delivery>,
  task: Option<JoinHandle<()>>,
}

impl Executor {
  pub(crate) fn new(timeout: Option<u32>) -> Self {
    Self {
      delivery: Arc::new(Delivery::new(timeout)),
      task: None,
    }
  }

  pub(crate) async fn close(&mut self) {
    *self.delivery.lock() = DeliveryState::default();
    self.stop_handler().await;
  }

  async fn stop_handler(&mut self) {
    if let Some(task) = self.task.take() {
      task.abort();
      let _ = task.await;
    }
  }

  pub(crate) async fn watch(
    &mut self,
    aggregate: Box<dyn EventAggregateHandler + Send>,
    raw: Box<dyn EventHandler + Send>,
  ) {
    self.stop_handler().await;
    let (tx, rx) = mpsc::unbounded_channel();
    self.delivery.lock().handler = Some(tx);
    self.task = Some(tokio::spawn(run(
      Arc::clone(&self.delivery),
      rx,
      aggregate,
      raw,
    )));
  }
}

async fn run(
  delivery: Arc<Delivery>,
  mut rx: mpsc::UnboundedReceiver<Message>,
  aggregate: Box<dyn EventAggregateHandler + Send>,
  raw: Box<dyn EventHandler + Send>,
) {
  loop {
    let deadline = delivery.lock().deadline;
    let events = if let Some(deadline) = deadline {
      tokio::select! {
        biased;
        events = rx.recv() => events,
        _ = sleep_until(deadline) => {
          if let Some(files) = delivery.take_due_batch(deadline, &rx) {
            aggregate.on_event_handle(files.changed, files.deleted);
          }
          continue;
        }
      }
    } else {
      rx.recv().await
    };
    let Some(message) = events else { return };
    let Message::Events(events) = message else {
      continue;
    };
    if delivery.lock().paused {
      continue;
    }
    deliver_raw(&*raw, events);
  }
}

fn deliver_raw(handler: &dyn EventHandler, events: ResolvedEvents) {
  for event in events.raw.into_iter().chain(events.recovered) {
    let path = event.path.to_string_lossy().into_owned();
    let result = match event.kind {
      FsEventKind::Change | FsEventKind::Create => handler.on_change(path),
      FsEventKind::Remove => handler.on_delete(path),
    };
    if result.is_err() {
      return;
    }
  }
  if let Some(path) = events.context_removal {
    let _ = handler.on_context_remove(path.to_string_lossy().into_owned());
  }
}

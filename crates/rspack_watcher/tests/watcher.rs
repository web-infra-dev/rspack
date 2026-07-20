#![allow(dead_code)]

use std::sync::atomic::AtomicU8;

use rspack_paths::ArcPath;
use rspack_watcher::{FsWatcher, FsWatcherOptions};

mod helpers;

macro_rules! e {
  () => {
    (std::iter::empty(), std::iter::empty())
  };
}

macro_rules! f {
  ($($file:expr),*) => {
    (vec![$(ArcPath::from($file)),*].into_iter(), std::iter::empty())
  };
}

macro_rules! h {
  ($options:expr) => {
    h!($options, Default::default())
  };
  ($options:expr, $ignore:expr) => {
    helpers::TestHelper::new(|| FsWatcher::new($options, $ignore))
  };
}

macro_rules! c {
  () => {
    AtomicU8::new(0)
  };
}

macro_rules! add {
  ($c:expr) => {
    $c.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
  };
}

macro_rules! load {
  ($c:expr) => {
    $c.load(std::sync::atomic::Ordering::SeqCst)
  };
}

#[allow(unused_macro_rules)]
macro_rules! watch {
  ($helper:expr, $($files:expr),*) => {
    watch!(files @ $helper, $($files),*)
  };
  ($helper:expr, _, $($dirs:expr),*) => {
    watch!(dirs @ $helper, $($dirs),*)
  };
  ($helper:expr, _, _, $($missing:expr),*) => {
    watch!(missing @ $helper, $($missing),*)
  };

  (files @ $helper:expr, $($files:expr),*) => {
    $helper.watch(f!($($files),*), e!(), e!())
  };
  (dirs @ $helper:expr, $($dirs:expr),*) => {
    $helper.watch(e!(), f!($($dirs),*), e!())
  };
  (missing @ $helper:expr, $($missing:expr),*) => {
    $helper.watch(e!(), e!(), f!($($missing),*))
  };
}

#[test]
fn should_watch_a_single_file() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(1000),
    ..Default::default()
  });

  let rx = watch!(helper, "a");

  helper.tick(|| {
    helper.file("a");
  });

  let change_events = c!();
  helper.collect_events(
    rx,
    |file, _| {
      file.assert_path(helper.join("a"));
      add!(change_events);
    },
    |changes, abort| {
      changes.assert_changed(helper.join("a"));
      assert!(load!(change_events) > 0);
      *abort = true;
    },
  );
}

#[tokio::test]
async fn should_report_error_when_watching_after_close() {
  use rspack_watcher::{EventAggregateHandler, EventHandler};

  struct ErrorProbe(std::sync::mpsc::Sender<String>);
  impl EventAggregateHandler for ErrorProbe {
    fn on_event_handle(
      &self,
      _changed_files: rspack_util::fx_hash::FxHashSet<String>,
      _deleted_files: rspack_util::fx_hash::FxHashSet<String>,
    ) {
    }
    fn on_error(&self, error: rspack_error::Error) {
      let _ = self.0.send(error.to_string());
    }
  }

  struct NoopHandler;
  impl EventHandler for NoopHandler {}

  let watcher = FsWatcher::new(FsWatcherOptions::default(), Default::default());
  watcher.close().await.unwrap();

  let (tx, rx) = std::sync::mpsc::channel();
  watcher
    .watch(
      e!(),
      e!(),
      e!(),
      std::time::SystemTime::now(),
      Box::new(ErrorProbe(tx)),
      Box::new(NoopHandler),
    )
    .await;

  let message = rx
    .try_recv()
    .expect("watch on a stopped watcher must be rejected through on_error");
  assert!(message.contains("stopped"), "unexpected message: {message}");
}

#[test]
fn should_emit_remove_when_a_watched_file_is_deleted() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(1000),
    ..Default::default()
  });

  // The file must exist before watching so its deletion is observed.
  helper.file("a");

  let rx = watch!(helper, "a");

  helper.tick(|| {
    std::fs::remove_file(helper.join("a")).unwrap();
  });

  let delete_events = c!();
  helper.collect_events(
    rx,
    |event, _| {
      // The initial scan and macOS FSEvents can emit an unrelated `change` for
      // `a` around the deletion; we only require that the deletion itself
      // surfaces as a `remove`, not a `change`.
      if let helpers::ChangedEvent::Deleted(_) = event {
        event.assert_path(helper.join("a"));
        add!(delete_events);
      }
    },
    |changes, abort| {
      changes.assert_deleted(helper.join("a"));
      assert!(load!(delete_events) > 0);
      *abort = true;
    },
  );
}

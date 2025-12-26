#[macro_export]
macro_rules! empty {
  () => {
    (std::iter::empty(), std::iter::empty())
  };
}

#[macro_export]
macro_rules! paths {
  ($($path:expr),*) => {
    (vec![$(rspack_paths::ArcPath::from($path)),*].into_iter(), std::iter::empty())
  };
}

#[macro_export]
macro_rules! helper {
  ($options:expr) => {
    helper!($options, Default::default())
  };
  ($options:expr, $ignore:expr) => {
    $crate::helpers::TestHelper::new(|| rspack_watcher::FsWatcher::new($options, $ignore))
  };
}

#[macro_export]
macro_rules! counter {
  () => {
    std::sync::atomic::AtomicU8::new(0)
  };
}

#[macro_export]
macro_rules! arc_counter {
  () => {
    std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0))
  };
}

#[macro_export]
macro_rules! add {
  ($c:expr) => {
    $c.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
  };
}

#[macro_export]
macro_rules! load {
  ($c:expr) => {
    $c.load(std::sync::atomic::Ordering::SeqCst)
  };
}

#[macro_export]
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
    $helper.watch(paths!($($files),*), empty!(), empty!())
  };
  (dirs @ $helper:expr, $($dirs:expr),*) => {
    $helper.watch(empty!(), paths!($($dirs),*), empty!())
  };
  (missing @ $helper:expr, $($missing:expr),*) => {
    $helper.watch(empty!(), empty!(), paths!($($missing),*))
  };
}

#[macro_export]
macro_rules! assert_no_events {
  ($change_events:expr, $aggregated_events:expr) => {
    assert_eq!(
      load!($change_events),
      0,
      "Expected no change events but got {}",
      load!($change_events)
    );
    assert_eq!(
      load!($aggregated_events),
      0,
      "Expected no aggregated events but got {}",
      load!($aggregated_events)
    );
  };
}

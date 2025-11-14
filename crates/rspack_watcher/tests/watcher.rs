#![allow(dead_code)]

use rspack_regex::RspackRegex;
use rspack_watcher::{FsWatcherIgnored, FsWatcherOptions};

mod helpers;

#[test]
fn should_watch_a_single_file() {
  let mut h = helper!(FsWatcherOptions {
    aggregate_timeout: Some(1000),
    ..Default::default()
  });

  let rx = watch!(h, "a");

  h.tick(|| {
    h.file("a");
  });

  let change_events = counter!();

  h.collect_events_blocking(
    rx,
    |file, _| {
      file.assert_path(h.join("a"));
      add!(change_events);
    },
    |changes, abort| {
      changes.assert_changed(h.join("a"));
      assert!(load!(change_events) > 0);
      *abort = true;
    },
  );
}

#[test]
fn should_not_watch_a_single_ignored_file_glob() {
  let mut h = helper!(
    FsWatcherOptions {
      aggregate_timeout: Some(300),
      ..Default::default()
    },
    FsWatcherIgnored::Path("**/a".to_string())
  );

  let rx = watch!(h, "a");

  let change_events = arc_counter!();
  let aggregated_events = arc_counter!();

  h.tick(|| {
    h.file("a");
    h.tick_ms(1000, || {
      {
        let change_events = change_events.clone();
        let aggregated_events = aggregated_events.clone();

        h.collect_events(
          rx,
          move |_file, _abort| {
            add!(change_events);
          },
          move |_changes, _abort| {
            add!(aggregated_events);
          },
        );
      }

      assert_no_events!(change_events, aggregated_events);
    });
  });
}

#[test]
fn should_not_watch_a_single_ignored_file_regexp() {
  let mut h = helper!(
    FsWatcherOptions {
      aggregate_timeout: Some(300),
      ..Default::default()
    },
    FsWatcherIgnored::Regex(RspackRegex::new(r"/a$").unwrap())
  );

  let rx = watch!(h, "a");

  let change_events = arc_counter!();
  let aggregated_events = arc_counter!();

  h.tick(|| {
    h.file("a");
    h.tick_ms(1000, || {
      {
        let change_events = change_events.clone();
        let aggregated_events = aggregated_events.clone();

        h.collect_events(
          rx,
          move |_file, _abort| {
            add!(change_events);
          },
          move |_changes, _abort| {
            add!(aggregated_events);
          },
        );
      }

      assert_no_events!(change_events, aggregated_events);
    });
  });
}

#[test]
#[ignore]
fn should_not_watch_a_single_ignored_file_function() {
  todo!("FsWatcherIgnored::Function")
}

#[test]
fn should_watch_multiple_files() {
  let mut h = helper!(FsWatcherOptions {
    aggregate_timeout: Some(1000),
    ..Default::default()
  });

  let rx = watch!(h, "a", "b");

  let change_events = arc_counter!();
  let changed_files = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

  h.tick_ms(400, || {
    h.file("a");
    h.tick_ms(400, || {
      h.file("b");
      h.tick_ms(400, || {
        h.file("a");
        h.tick_ms(400, || {
          h.file("b");
          h.tick_ms(400, || {
            h.file("a");
          });
        });
      });
    });
  });

  h.collect_events_blocking(
    rx,
    {
      let change_events = change_events.clone();
      let changed_files = changed_files.clone();
      move |file, _| {
        let mut files = changed_files.lock().unwrap();
        let file_str = match file {
          helpers::ChangedEvent::Changed(path) => path.clone(),
          _ => return,
        };
        // Dedupe consecutive duplicates
        if files.is_empty() || files.last().unwrap() != &file_str {
          files.push(file_str);
        }
        add!(change_events);
      }
    },
    {
      let change_events = change_events.clone();
      let changed_files = changed_files.clone();
      let a = h.join("a").to_string();
      let b = h.join("b").to_string();
      move |changes, abort| {
        // Verify aggregated changes contain both files
        let mut sorted_changes: Vec<String> = changes.changed_files.iter().cloned().collect();
        sorted_changes.sort();
        assert_eq!(sorted_changes, vec![a.clone(), b.clone()]);

        // Verify the change event sequence
        let files = changed_files.lock().unwrap();
        assert_eq!(
          *files,
          vec![a.clone(), b.clone(), a.clone(), b.clone(), a.clone()]
        );

        assert!(load!(change_events) > 0);
        *abort = true;
      }
    },
  );
}

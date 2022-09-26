use std::path::PathBuf;

use rspack_core::log::enable_tracing_by_env;
use rspack_test::{read_test_config_and_normalize, test_fixture};
use testing_macros::fixture;

// #[fixture("tests/fixtures/*")]
// fn rspack(fixture_path: PathBuf) {
//   enable_tracing_by_env();
//   test_fixture(&fixture_path);
// }

// #[tokio::main]
// async fn run(context: PathBuf) {
//   let options = read_test_config_and_normalize(&context);
//   let mut compiler = rspack::rspack(options, vec![]);
//   compiler.run().await.unwrap();
// }

// #[fixture("../../examples/*")]
// fn example(fixture_path: PathBuf) {
//   run(fixture_path);
// }

#[test]
fn test_lodash() {
  // span_tree().aggregate(true).enable();
  let start = std::time::SystemTime::now();
  let path = std::path::Path::new("/Users/bytedance/rspack/benchcases/lodash-with-simple-css");
  let options = read_test_config_and_normalize(&path);
  let mut compiler = rspack::rspack(options, vec![]);
  compiler.run().unwrap();
  println!("cost: {:?}", start.elapsed());
}

use std::{
  fmt, mem,
  time::{Duration, Instant},
};

use tracing::{
  debug,
  field::{Field, Visit},
  span::Attributes,
  Event, Id, Subscriber,
};
use tracing_subscriber::{
  layer::Context,
  prelude::*,
  registry::{LookupSpan, Registry},
  Layer,
};

pub fn span_tree() -> SpanTree {
  SpanTree::default()
}

#[derive(Default)]
pub struct SpanTree {
  aggregate: bool,
}

impl SpanTree {
  /// Merge identical sibling spans together.
  pub fn aggregate(self, yes: bool) -> SpanTree {
    SpanTree {
      aggregate: yes,
      ..self
    }
  }
  /// Set as a global subscriber
  pub fn enable(self) {
    let subscriber = Registry::default().with(self);
    tracing::subscriber::set_global_default(subscriber)
      .unwrap_or_else(|_| debug!("Global subscriber is already set"));
  }
}

struct Data {
  start: Instant,
  children: Vec<Node>,
}

impl Data {
  fn new(attrs: &Attributes<'_>) -> Self {
    let mut span = Self {
      start: Instant::now(),
      children: Vec::new(),
    };
    attrs.record(&mut span);
    span
  }
  fn into_node(self, name: &'static str) -> Node {
    Node {
      name,
      count: 1,
      duration: self.start.elapsed(),
      children: self.children,
    }
  }
}

impl Visit for Data {
  fn record_debug(&mut self, _field: &Field, _value: &dyn fmt::Debug) {}
}

impl<S> Layer<S> for SpanTree
where
  S: Subscriber + for<'span> LookupSpan<'span> + fmt::Debug,
{
  fn on_new_span(&self, attrs: &Attributes, id: &Id, ctx: Context<S>) {
    let span = ctx.span(id).unwrap();

    let data = Data::new(attrs);
    span.extensions_mut().insert(data);
  }

  fn on_event(&self, _event: &Event<'_>, _ctx: Context<S>) {}

  fn on_close(&self, id: Id, ctx: Context<S>) {
    let span = ctx.span(&id).unwrap();
    let data = span.extensions_mut().remove::<Data>().unwrap();
    let mut node = data.into_node(span.name());

    match span.parent() {
      Some(parent_span) => {
        parent_span
          .extensions_mut()
          .get_mut::<Data>()
          .unwrap()
          .children
          .push(node);
      }
      None => {
        if self.aggregate {
          node.aggregate()
        }
        node.print()
      }
    }
  }
}

#[derive(Default)]
struct Node {
  name: &'static str,
  count: u32,
  duration: Duration,
  children: Vec<Node>,
}

impl Node {
  fn print(&self) {
    self.go(0)
  }
  fn go(&self, level: usize) {
    if level == 0 && self.name != "resolve" {
      return;
    }
    if level == 0 && self.duration < Duration::from_millis(3) {
      return;
    }
    let bold = "\u{001b}[1m";
    let reset = "\u{001b}[0m";

    let duration = format!("{:3.2?}", self.duration);
    let count = if self.count > 1 {
      self.count.to_string()
    } else {
      String::new()
    };
    eprintln!(
      "{:width$}  {:<9} {:<6} {bold}{}{reset}",
      "",
      duration,
      count,
      self.name,
      bold = bold,
      reset = reset,
      width = level * 2
    );
    for child in &self.children {
      child.go(level + 1)
    }
    if level == 0 {
      eprintln!()
    }
  }

  fn aggregate(&mut self) {
    if self.children.is_empty() {
      return;
    }

    self.children.sort_by_key(|it| it.name);
    let mut idx = 0;
    for i in 1..self.children.len() {
      if self.children[idx].name == self.children[i].name {
        let child = mem::take(&mut self.children[i]);
        self.children[idx].duration += child.duration;
        self.children[idx].count += child.count;
        self.children[idx].children.extend(child.children);
      } else {
        idx += 1;
        assert!(idx <= i);
        self.children.swap(idx, i);
      }
    }
    self.children.truncate(idx + 1);
    for child in &mut self.children {
      child.aggregate()
    }
  }
}

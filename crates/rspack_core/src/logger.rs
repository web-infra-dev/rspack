use std::{
  backtrace::Backtrace,
  time::{Duration, Instant},
};

use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub enum Log {
  Error {
    message: String,
    trace: Vec<String>,
  },
  Warn {
    message: String,
    trace: Vec<String>,
  },
  Info {
    message: String,
  },
  Log {
    message: String,
  },
  Debug {
    message: String,
  },
  Trace {
    message: String,
    trace: Vec<String>,
  },
  Group {
    message: String,
  },
  GroupCollapsed {
    message: String,
  },
  GroupEnd,
  Profile {
    label: &'static str,
  },
  ProfileEnd {
    label: &'static str,
  },
  Time {
    label: &'static str,
    secs: u64,
    subsec_nanos: u32,
  },
  Clear,
  Status {
    message: String,
  },
  Cache {
    label: &'static str,
    hit: u32,
    total: u32,
  },
}

impl Log {
  pub fn to_bit_flag(&self) -> u32 {
    match self {
      Log::Error { .. } => 1 << 0,
      Log::Warn { .. } => 1 << 1,
      Log::Info { .. } => 1 << 2,
      Log::Log { .. } => 1 << 3,
      Log::Debug { .. } => 1 << 4,
      Log::Trace { .. } => 1 << 5,
      Log::Group { .. } => 1 << 6,
      Log::GroupCollapsed { .. } => 1 << 7,
      Log::GroupEnd => 1 << 8,
      Log::Profile { .. } => 1 << 9,
      Log::ProfileEnd { .. } => 1 << 10,
      Log::Time { .. } => 1 << 11,
      Log::Clear => 1 << 12,
      Log::Status { .. } => 1 << 13,
      Log::Cache { .. } => 1 << 14,
    }
  }
}

fn capture_trace() -> Vec<String> {
  Backtrace::force_capture()
    .to_string()
    .split('\n')
    .enumerate()
    .filter(|(i, _)| i % 2 != 0) // even line is function name, odd line is code position, only need code positiion
    .skip(5) // remove some useless lines
    .take(8)
    .map(|(_, line)| line[9..].to_owned()) // remove some empty chars
    .collect()
}

pub trait Logger {
  fn raw(&mut self, log_type: Log);

  fn error(&mut self, message: impl Into<String>) {
    self.raw(Log::Error {
      message: message.into(),
      trace: capture_trace(),
    })
  }

  fn warn(&mut self, message: impl Into<String>) {
    self.raw(Log::Warn {
      message: message.into(),
      trace: capture_trace(),
    })
  }

  fn info(&mut self, message: impl Into<String>) {
    self.raw(Log::Info {
      message: message.into(),
    })
  }

  fn log(&mut self, message: impl Into<String>) {
    self.raw(Log::Log {
      message: message.into(),
    })
  }

  fn debug(&mut self, message: impl Into<String>) {
    self.raw(Log::Debug {
      message: message.into(),
    })
  }

  fn assert(&mut self, assertion: bool, message: impl Into<String>) {
    if !assertion {
      self.error(message);
    }
  }

  fn trace(&mut self) {
    self.raw(Log::Trace {
      message: "Trace".to_string(),
      trace: capture_trace(),
    })
  }

  fn clear(&mut self) {
    self.raw(Log::Clear)
  }

  fn status(&mut self, message: impl Into<String>) {
    self.raw(Log::Status {
      message: message.into(),
    })
  }

  fn profile(&mut self, label: &'static str) {
    self.raw(Log::Profile { label })
  }

  fn profile_end(&mut self, label: &'static str) {
    self.raw(Log::ProfileEnd { label })
  }

  fn group(&mut self, message: impl Into<String>) {
    self.raw(Log::Group {
      message: message.into(),
    })
  }

  fn group_collapsed(&mut self, message: impl Into<String>) {
    self.raw(Log::GroupCollapsed {
      message: message.into(),
    })
  }

  fn group_end(&mut self) {
    self.raw(Log::GroupEnd)
  }

  fn time(&self, label: &'static str) -> StartTime {
    StartTime {
      label,
      start: Instant::now(),
    }
  }

  fn time_log(&mut self, start: &StartTime) {
    let elapsed = start.elapsed();
    let secs = elapsed.as_secs();
    let subsec_nanos = elapsed.subsec_nanos();
    self.raw(Log::Time {
      label: start.label,
      secs,
      subsec_nanos,
    })
  }

  fn time_end(&mut self, start: StartTime) {
    self.time_log(&start)
  }

  fn time_aggregate(&self, label: &'static str) -> StartTimeAggregate {
    StartTimeAggregate {
      duration: Duration::ZERO,
      label,
    }
  }

  fn time_aggregate_end(&mut self, start: StartTimeAggregate) {
    let secs = start.duration.as_secs();
    let subsec_nanos = start.duration.subsec_nanos();
    self.raw(Log::Time {
      label: start.label,
      secs,
      subsec_nanos,
    })
  }

  fn cache(&self, label: &'static str) -> CacheCount {
    CacheCount {
      label,
      total: 0,
      hit: 0,
    }
  }

  fn cache_end(&mut self, count: CacheCount) {
    if count.total != 0 {
      self.raw(Log::Cache {
        label: count.label,
        hit: count.hit,
        total: count.total,
      })
    }
  }
}

pub struct StartTime {
  label: &'static str,
  start: Instant,
}

impl StartTime {
  pub fn elapsed(&self) -> Duration {
    self.start.elapsed()
  }
}

pub struct StartTimeAggregate {
  duration: Duration,
  label: &'static str,
}

impl StartTimeAggregate {
  pub fn start(&self) -> StartTime {
    StartTime {
      label: self.label,
      start: Instant::now(),
    }
  }

  pub fn end(&mut self, start: StartTime) {
    if start.label == self.label {
      self.duration += start.elapsed();
    } else {
      panic!(
        "label for StartTimeAggregate should be the same, expect: {}, actual: {}",
        self.label, start.label
      );
    }
  }
}

#[derive(Debug)]
pub struct CacheCount {
  label: &'static str,
  hit: u32,
  total: u32,
}

impl CacheCount {
  pub fn hit(&mut self) {
    self.total += 1;
    self.hit += 1;
  }

  pub fn miss(&mut self) {
    self.total += 1;
  }
}

#[derive(Debug, Default)]
pub struct CompilationLogging {
  loggers: FxHashMap<String, Logs>,
}

impl CompilationLogging {
  pub fn collect_logger(&mut self, logger: CompilationLogger) {
    self
      .loggers
      .entry(logger.name)
      .or_default()
      .extend(logger.logs);
  }

  pub fn iter(&self) -> impl Iterator<Item = (&String, &Logs)> {
    self.loggers.iter()
  }
}

#[derive(Debug)]
pub struct CompilationLogger {
  logs: Logs,
  name: String,
}

impl CompilationLogger {
  pub fn new(name: String) -> Self {
    Self {
      logs: Default::default(),
      name,
    }
  }

  pub fn collect_logs(&mut self, logs: Logs) {
    self.logs.extend(logs);
  }
}

impl Logger for CompilationLogger {
  fn raw(&mut self, log_type: Log) {
    self.logs.raw(log_type);
  }
}

#[derive(Debug, Default, Clone)]
pub struct Logs {
  inner: Vec<Log>,
}

impl Logger for Logs {
  fn raw(&mut self, log_type: Log) {
    self.inner.push(log_type);
  }
}

impl IntoIterator for Logs {
  type Item = Log;
  type IntoIter = std::vec::IntoIter<Log>;

  fn into_iter(self) -> Self::IntoIter {
    self.inner.into_iter()
  }
}

impl Extend<Log> for Logs {
  fn extend<T: IntoIterator<Item = Log>>(&mut self, iter: T) {
    self.inner.extend(iter)
  }
}

use std::{
  backtrace::Backtrace,
  fmt,
  hash::BuildHasherDefault,
  sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
  },
  time::{Duration, Instant},
};

use dashmap::DashMap;
use rustc_hash::FxHasher;

#[derive(Debug, Clone)]
pub enum LogType {
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

impl LogType {
  pub fn to_bit_flag(&self) -> u32 {
    match self {
      LogType::Error { .. } => 1 << 0,
      LogType::Warn { .. } => 1 << 1,
      LogType::Info { .. } => 1 << 2,
      LogType::Log { .. } => 1 << 3,
      LogType::Debug { .. } => 1 << 4,
      LogType::Trace { .. } => 1 << 5,
      LogType::Group { .. } => 1 << 6,
      LogType::GroupCollapsed { .. } => 1 << 7,
      LogType::GroupEnd => 1 << 8,
      LogType::Profile { .. } => 1 << 9,
      LogType::ProfileEnd { .. } => 1 << 10,
      LogType::Time { .. } => 1 << 11,
      LogType::Clear => 1 << 12,
      LogType::Status { .. } => 1 << 13,
      LogType::Cache { .. } => 1 << 14,
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
    .filter_map(|(_, line)| line.get(9..).map(|x| x.to_string())) // remove some empty chars
    .collect()
}

pub trait Logger {
  fn raw(&self, log_type: LogType);

  fn error(&self, message: impl Into<String>) {
    self.raw(LogType::Error {
      message: message.into(),
      trace: capture_trace(),
    })
  }

  fn warn(&self, message: impl Into<String>) {
    self.raw(LogType::Warn {
      message: message.into(),
      trace: capture_trace(),
    })
  }

  fn info(&self, message: impl Into<String>) {
    self.raw(LogType::Info {
      message: message.into(),
    })
  }

  fn log(&self, message: impl Into<String>) {
    self.raw(LogType::Log {
      message: message.into(),
    })
  }

  fn debug(&self, message: impl Into<String>) {
    self.raw(LogType::Debug {
      message: message.into(),
    })
  }

  fn assert(&self, assertion: bool, message: impl Into<String>) {
    if !assertion {
      self.error(message);
    }
  }

  fn trace(&self) {
    self.raw(LogType::Trace {
      message: "Trace".to_string(),
      trace: capture_trace(),
    })
  }

  fn clear(&self) {
    self.raw(LogType::Clear)
  }

  fn status(&self, message: impl Into<String>) {
    self.raw(LogType::Status {
      message: message.into(),
    })
  }

  fn profile(&self, label: &'static str) {
    self.raw(LogType::Profile { label })
  }

  fn profile_end(&self, label: &'static str) {
    self.raw(LogType::ProfileEnd { label })
  }

  fn group(&self, message: impl Into<String>) {
    self.raw(LogType::Group {
      message: message.into(),
    })
  }

  fn group_collapsed(&self, message: impl Into<String>) {
    self.raw(LogType::GroupCollapsed {
      message: message.into(),
    })
  }

  fn group_end(&self) {
    self.raw(LogType::GroupEnd)
  }

  fn time(&self, label: &'static str) -> StartTime {
    StartTime {
      label,
      start: Instant::now(),
    }
  }

  fn time_log(&self, start: &StartTime) {
    let elapsed = start.elapsed();
    let secs = elapsed.as_secs();
    let subsec_nanos = elapsed.subsec_nanos();
    self.raw(LogType::Time {
      label: start.label,
      secs,
      subsec_nanos,
    })
  }

  fn time_end(&self, start: StartTime) {
    self.time_log(&start)
  }

  fn time_aggregate(&self, label: &'static str) -> StartTimeAggregate {
    StartTimeAggregate {
      duration: Duration::ZERO,
      label,
    }
  }

  fn time_aggregate_end(&self, start: StartTimeAggregate) {
    let secs = start.duration.as_secs();
    let subsec_nanos = start.duration.subsec_nanos();
    self.raw(LogType::Time {
      label: start.label,
      secs,
      subsec_nanos,
    })
  }

  fn cache(&self, label: &'static str) -> CacheCount {
    CacheCount {
      label,
      total: AtomicU32::new(0),
      hit: AtomicU32::new(0),
    }
  }

  fn cache_end(&self, count: CacheCount) {
    let total = count.total.load(Ordering::Relaxed);
    if total != 0 {
      self.raw(LogType::Cache {
        label: count.label,
        hit: count.hit.load(Ordering::Relaxed),
        total,
      })
    }
  }
}

#[derive(Debug, Clone)]
pub struct InfrastructureLogEvent {
  pub name: Arc<str>,
  pub log_type: LogType,
}

pub trait InfrastructureLogSink: Send + Sync {
  fn emit(&self, event: InfrastructureLogEvent);
}

#[derive(Clone)]
pub struct InfrastructureLogger {
  sink: Arc<dyn InfrastructureLogSink>,
  name: Arc<str>,
}

impl fmt::Debug for InfrastructureLogger {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("InfrastructureLogger")
      .field("name", &self.name)
      .finish_non_exhaustive()
  }
}

impl InfrastructureLogger {
  pub fn new(name: impl Into<Arc<str>>, sink: Arc<dyn InfrastructureLogSink>) -> Self {
    Self {
      sink,
      name: name.into(),
    }
  }

  pub fn get_child(&self, name: &str) -> Self {
    let mut child_name = self.name.to_string();
    child_name.push('/');
    child_name.push_str(name);
    Self {
      sink: self.sink.clone(),
      name: Arc::from(child_name),
    }
  }
}

impl Logger for InfrastructureLogger {
  fn raw(&self, log_type: LogType) {
    self.sink.emit(InfrastructureLogEvent {
      name: self.name.clone(),
      log_type,
    });
  }
}

#[derive(Debug, Default)]
pub struct PrintlnInfrastructureLogSink;

impl InfrastructureLogSink for PrintlnInfrastructureLogSink {
  fn emit(&self, event: InfrastructureLogEvent) {
    let name = event.name;
    match event.log_type {
      LogType::Error { message, .. }
      | LogType::Warn { message, .. }
      | LogType::Info { message }
      | LogType::Log { message }
      | LogType::Debug { message }
      | LogType::Trace { message, .. } => println!("[{name}] {message}"),
      LogType::Group { message } | LogType::GroupCollapsed { message } => {
        println!("[{name}] {message}")
      }
      LogType::GroupEnd | LogType::Clear => {}
      LogType::Profile { label } => println!("[{name}] Profile {label}"),
      LogType::ProfileEnd { label } => println!("[{name}] Profile end {label}"),
      LogType::Time {
        label,
        secs,
        subsec_nanos,
      } => println!(
        "[{name}] {label}: {} ms",
        secs as f64 * 1000.0 + subsec_nanos as f64 / 1_000_000.0
      ),
      LogType::Status { message } => println!("[{name}] {message}"),
      LogType::Cache { label, hit, total } => println!(
        "[{name}] {label}: {:.1}% ({hit}/{total})",
        if total == 0 {
          0.0
        } else {
          hit as f32 / total as f32 * 100.0
        }
      ),
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

impl StartTimeAggregate {}

#[derive(Debug)]
pub struct CacheCount {
  label: &'static str,
  hit: AtomicU32,
  total: AtomicU32,
}

impl CacheCount {
  pub fn hit(&self) {
    self.total.fetch_add(1, Ordering::Relaxed);
    self.hit.fetch_add(1, Ordering::Relaxed);
  }

  pub fn miss(&self) {
    self.total.fetch_add(1, Ordering::Relaxed);
  }
}

pub type CompilationLogging = Arc<DashMap<Arc<str>, Vec<LogType>, BuildHasherDefault<FxHasher>>>;

#[derive(Clone)]
pub struct CompilationLogger {
  logging: CompilationLogging,
  name: Arc<str>,
}

impl fmt::Debug for CompilationLogger {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CompilationLogger")
      .field("name", &self.name)
      .finish_non_exhaustive()
  }
}

impl CompilationLogger {
  pub fn new(name: impl Into<Arc<str>>, logging: CompilationLogging) -> Self {
    Self {
      logging,
      name: name.into(),
    }
  }

  pub fn get_child(&self, name: &str) -> Self {
    let mut child_name = self.name.to_string();
    child_name.push('/');
    child_name.push_str(name);
    Self {
      logging: self.logging.clone(),
      name: Arc::from(child_name),
    }
  }
}

impl Logger for CompilationLogger {
  fn raw(&self, log_type: LogType) {
    if let Some(mut value) = self.logging.get_mut(&self.name) {
      value.push(log_type);
    } else {
      self.logging.insert(self.name.clone(), vec![log_type]);
    }
  }
}

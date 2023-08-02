use std::{
  backtrace::Backtrace,
  time::{Duration, Instant},
};

use crossbeam_channel::Sender;

#[derive(Debug)]
pub enum LogType {
  Error {
    message: String,
    trace: Backtrace,
  },
  Warn {
    message: String,
    trace: Backtrace,
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
    trace: Backtrace,
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
}

pub trait Logger {
  fn raw(&self, log_type: LogType);

  fn error(&self, message: String) {
    self.raw(LogType::Error {
      message,
      trace: Backtrace::force_capture(),
    })
  }

  fn warn(&self, message: String) {
    self.raw(LogType::Warn {
      message,
      trace: Backtrace::force_capture(),
    })
  }

  fn info(&self, message: String) {
    self.raw(LogType::Info { message })
  }

  fn log(&self, message: String) {
    self.raw(LogType::Log { message })
  }

  fn debug(&self, message: String) {
    self.raw(LogType::Debug { message })
  }

  fn assert(&self, assertion: bool, message: String) {
    if !assertion {
      self.error(message);
    }
  }

  fn trace(&self) {
    self.raw(LogType::Trace {
      trace: Backtrace::force_capture(),
    })
  }

  fn clear(&self) {
    self.raw(LogType::Clear)
  }

  fn status(&self, message: String) {
    self.raw(LogType::Status { message })
  }

  fn profile(&self, label: &'static str) {
    self.raw(LogType::Profile { label })
  }

  fn profile_end(&self, label: &'static str) {
    self.raw(LogType::ProfileEnd { label })
  }

  fn group(&self, message: String) {
    self.raw(LogType::Group { message })
  }

  fn group_collapsed(&self, message: String) {
    self.raw(LogType::GroupCollapsed { message })
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

pub struct CompilationLogger {
  tx: Sender<(String, LogType)>,
  name: String,
}

impl CompilationLogger {
  pub fn new(name: String, sender: Sender<(String, LogType)>) -> Self {
    Self { tx: sender, name }
  }
}

impl Logger for CompilationLogger {
  fn raw(&self, log_type: LogType) {
    self
      .tx
      .send((self.name.clone(), log_type))
      .expect("Logger::raw send message failed");
  }
}

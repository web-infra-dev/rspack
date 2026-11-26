use std::{
  collections::VecDeque,
  sync::{Arc, Weak},
};

use rspack_paths::InternedPath;

use crate::{
  EventBatch, FsEvent, FsEventKind, ResolvedEvents, executor::FilesData, paths::PathManager,
  trigger::Input,
};

#[derive(Clone)]
pub(super) enum BufferedEvent {
  Input {
    input: Arc<Input>,
    detected: bool,
    recovered: EventBatch,
  },
  Scan {
    events: Arc<ResolvedEvents>,
    subscribers: Arc<[Weak<InternedPath>]>,
  },
}

struct Pending {
  event: BufferedEvent,
  visible: EventBatch,
  consumed: EventBatch,
}

#[derive(Default)]
pub(super) struct PausedEvents {
  events: VecDeque<Pending>,
}

impl PausedEvents {
  pub(super) fn push(
    &mut self,
    input: Option<Input>,
    events: Option<ResolvedEvents>,
    subscribers: Vec<Weak<InternedPath>>,
  ) {
    let visible = events
      .as_ref()
      .map_or_else(Vec::new, |e| e.dependencies.clone());
    let event = if let Some(input) = input {
      BufferedEvent::Input {
        input: Arc::new(input),
        detected: events.is_some(),
        recovered: events.map_or_else(Vec::new, |e| e.recovered),
      }
    } else if let Some(events) = events {
      BufferedEvent::Scan {
        events: Arc::new(events),
        subscribers: subscribers.into(),
      }
    } else {
      return;
    };
    self.events.push_back(Pending {
      event,
      visible,
      consumed: Vec::new(),
    });
  }

  pub(super) fn front(&self) -> Option<BufferedEvent> {
    Some(self.events.front()?.event.clone())
  }

  pub(super) fn project(&mut self, files: &mut FilesData, take: bool) {
    for pending in &mut self.events {
      files.record(&pending.visible);
      if take {
        pending.consumed.append(&mut pending.visible);
      }
    }
  }

  pub(super) fn retain_subscriptions(&mut self, paths: &PathManager) {
    for pending in &mut self.events {
      pending.visible.retain(|e| paths.is_registered(&e.path));
    }
  }

  pub(super) fn interrupted(&mut self, observed: Option<ResolvedEvents>) {
    if let Some(events) = observed
      && let Some(Pending {
        event:
          BufferedEvent::Input {
            detected,
            recovered,
            ..
          },
        ..
      }) = self.events.front_mut()
    {
      *detected = true;
      for event in events.recovered {
        if !recovered.iter().any(|old| old.path == event.path) {
          recovered.push(event);
        }
      }
    }
  }

  pub(super) fn finish(&mut self, mut events: Option<ResolvedEvents>) -> Option<ResolvedEvents> {
    let pending = self
      .events
      .pop_front()
      .expect("replaying the first pending input");
    if !pending.consumed.is_empty()
      && let Some(events) = &mut events
    {
      events.retain_dependencies(|e| {
        !pending.consumed.iter().any(|old| {
          old.path == e.path && (old.kind == FsEventKind::Remove) == (e.kind == FsEventKind::Remove)
        })
      });
    }
    events
  }
}

impl ResolvedEvents {
  pub(super) fn retain_dependencies(&mut self, keep: impl FnMut(&FsEvent) -> bool) {
    self.dependencies.retain(keep);
    let has_recipient = |path: &InternedPath| {
      self
        .dependencies
        .iter()
        .any(|dep| path.starts_with(dep.path.as_path()))
    };
    self.raw = self.raw.take().filter(|e| has_recipient(&e.path));
    self.recovered.retain(|e| has_recipient(&e.path));
    self.context_removal = self.context_removal.take().filter(has_recipient);
  }
}

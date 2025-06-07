use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicU64, Ordering},
    LazyLock, Mutex,
  },
};

use micromegas_perfetto::protos::{self as idl, TracePacket};

thread_local! {
    static THREAD_TRACK_UUID: AtomicU64 = AtomicU64::new(unique_uuid());
}
static CUSTOM_SCOPE_NAMES: LazyLock<Mutex<HashMap<String, u64>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Default)]
pub struct DebugAnnotations {
  pub annotations: Vec<idl::DebugAnnotation>,
}

static GLOBAL_COUNTER: AtomicU64 = AtomicU64::new(1);
pub fn unique_uuid() -> u64 {
  GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn current_thread_uuid() -> u64 {
  THREAD_TRACK_UUID.with(|id| id.load(Ordering::Relaxed))
}

pub fn create_track_descriptor(
  uuid: u64,
  parent_uuid: Option<u64>,
  name: Option<impl AsRef<str>>,
  process: Option<idl::ProcessDescriptor>,
  thread: Option<idl::ThreadDescriptor>,
  counter: Option<idl::CounterDescriptor>,
) -> idl::TrackDescriptor {
  let mut desc = idl::TrackDescriptor::default();

  desc.uuid = Some(uuid);
  desc.parent_uuid = parent_uuid;
  desc.static_or_dynamic_name = name
    .map(|s| s.as_ref().to_string())
    .map(idl::track_descriptor::StaticOrDynamicName::Name);
  desc.process = process;
  desc.thread = thread;
  desc.counter = counter;
  desc
}

pub fn create_event(
  track_uuid: u64,
  name: Option<&str>,
  location: Option<(&str, u32)>,
  debug_annotations: DebugAnnotations,
  r#type: Option<idl::track_event::Type>,
) -> idl::TrackEvent {
  let mut event = idl::TrackEvent::default();

  event.track_uuid = Some(track_uuid);
  event.categories = vec!["".to_string()];
  if let Some(name) = name {
    event.name_field = Some(idl::track_event::NameField::Name(name.to_string()));
  }
  if let Some(t) = r#type {
    event.set_type(t);
  }
  if !debug_annotations.annotations.is_empty() {
    event.debug_annotations = debug_annotations.annotations;
  }
  if let Some((file, line)) = location {
    let mut source_location = idl::SourceLocation::default();
    source_location.file_name = Some(file.to_owned());
    source_location.line_number = Some(line);
    let location = idl::track_event::SourceLocationField::SourceLocation(source_location);
    event.source_location_field = Some(location);
  }
  event
}

// write a custom scope to the trace, more info see https://perfetto.dev/docs/reference/synthetic-track-event#custom-scoped-slices
pub fn custom_scope_packet(scope_name: String) -> (Option<TracePacket>, u64) {
  if let Some(uuid) = CUSTOM_SCOPE_NAMES.lock().unwrap().get(&scope_name) {
    return (None, *uuid);
  }
  let uuid = unique_uuid();
  CUSTOM_SCOPE_NAMES
    .lock()
    .unwrap()
    .insert(scope_name.clone(), uuid);

  let track_descriptor = create_track_descriptor(
    uuid,              // uuid
    None,              // parent_uuid
    Some(&scope_name), // name
    None,              // process
    None,              // thread descriptor
    None,              // counter descriptor
  );
  let mut packet = idl::TracePacket::default();
  packet.data = Some(idl::trace_packet::Data::TrackDescriptor(track_descriptor));
  packet.sequence_flags = Some(1);

  (Some(packet), uuid)
}

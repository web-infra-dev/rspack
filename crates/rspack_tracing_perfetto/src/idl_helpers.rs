use std::{
  collections::HashMap,
  sync::{
    LazyLock, Mutex,
    atomic::{AtomicU64, Ordering},
  },
};

use micromegas_perfetto::protos::{self as idl, TracePacket};

#[cfg_attr(allocative, allocative::root)]
static CUSTOM_SCOPE_NAMES: LazyLock<Mutex<HashMap<String, u64>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Default)]
pub struct DebugAnnotations {
  pub annotations: Vec<idl::DebugAnnotation>,
}

static GLOBAL_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn unique_uuid() -> u64 {
  GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn create_track_descriptor(
  uuid: u64,
  parent_uuid: Option<u64>,
  name: Option<impl AsRef<str>>,
) -> idl::TrackDescriptor {
  idl::TrackDescriptor {
    uuid: Some(uuid),
    parent_uuid,
    static_or_dynamic_name: name
      .map(|s| s.as_ref().to_string())
      .map(idl::track_descriptor::StaticOrDynamicName::Name),
    ..Default::default()
  }
}

pub fn create_event(
  track_uuid: u64,
  name: Option<&str>,
  location: Option<(&str, u32)>,
  debug_annotations: DebugAnnotations,
  r#type: Option<idl::track_event::Type>,
) -> idl::TrackEvent {
  let mut event = idl::TrackEvent {
    track_uuid: Some(track_uuid),
    categories: vec![],
    ..Default::default()
  };
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
    let source_location = idl::SourceLocation {
      file_name: Some(file.to_owned()),
      line_number: Some(line),
      ..Default::default()
    };
    let location = idl::track_event::SourceLocationField::SourceLocation(source_location);
    event.source_location_field = Some(location);
  }
  event
}

// write a custom scope to the trace, more info see https://perfetto.dev/docs/reference/synthetic-track-event#custom-scoped-slices
// it will only create a packet if the scope name is not already registered
pub fn create_scope_sliced_packet(scope_name: String) -> (Option<TracePacket>, u64) {
  // allocate a new uuid for the scope name if it is not already registered
  if let Some(uuid) = CUSTOM_SCOPE_NAMES
    .lock()
    .expect("lock failed")
    .get(&scope_name)
  {
    return (None, *uuid);
  }
  let uuid = unique_uuid();
  CUSTOM_SCOPE_NAMES
    .lock()
    .expect("lock failed")
    .insert(scope_name.clone(), uuid);

  let track_descriptor = create_track_descriptor(
    uuid,              // uuid
    None,              // parent_uuid
    Some(&scope_name), // name
  );
  let packet = idl::TracePacket {
    data: Some(idl::trace_packet::Data::TrackDescriptor(track_descriptor)),
    ..Default::default()
  };

  (Some(packet), uuid)
}

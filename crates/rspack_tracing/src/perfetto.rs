use std::{collections::HashMap, io::Write};

use rspack_tracing_perfetto::{
  BytesMut, PerfettoLayer, idl,
  idl::TrackDescriptor,
  idl_helpers::{
    DebugAnnotations, create_event, create_scope_sliced_packet, create_track_descriptor,
    unique_uuid,
  },
  prost::Message,
};
static JAVASCRIPT_ANALYSIS_TRACK: &str = "JavaScript Analysis";
use crate::{Tracer, tracer::Layered};
#[derive(Default)]
pub struct PerfettoTracer {
  track_state: HashMap<u32, (TrackDescriptor, idl::Trace)>,
  event_id_map: HashMap<u32, u64>,
  writer: Option<std::fs::File>,
}
// convert hashmap to perfetto debug annotations
// the values are always json string
fn to_debug_annotation(map: Option<HashMap<String, String>>) -> DebugAnnotations {
  let mut annotations = DebugAnnotations::default();
  if let Some(map) = map {
    for (key, value) in map {
      let annotation = idl::DebugAnnotation {
        name_field: Some(idl::debug_annotation::NameField::Name(key)),
        value: Some(idl::debug_annotation::Value::LegacyJsonValue(value)),
        ..Default::default()
      };

      annotations.annotations.push(annotation);
    }
  }
  annotations
}
impl PerfettoTracer {
  // write the log and the related track descriptor to the writer
  fn write_log(&mut self, log: &mut idl::Trace, track_descriptor: Option<TrackDescriptor>) {
    let mut buf = BytesMut::new();
    if let Some(task_descriptor) = track_descriptor {
      let packet = idl::TracePacket {
        data: Some(idl::trace_packet::Data::TrackDescriptor(task_descriptor)),
        ..Default::default()
      };

      log.packet.insert(0, packet);
    }

    let Ok(_) = log.encode(&mut buf) else {
      return;
    };
    {
      let _ = self
        .writer
        .as_ref()
        .expect("should set writer first")
        .write_all(&buf);
    };
  }
}
impl Tracer for PerfettoTracer {
  fn setup(&mut self, output: &str) -> Option<Layered> {
    let trace_file = std::fs::File::create(output)
      .unwrap_or_else(|e| panic!("failed to create trace file: {output} due to {e}"));
    self.writer = trace_file.try_clone().ok();
    let layer = PerfettoLayer::new(trace_file).with_debug_annotations(true);
    Some(Box::new(layer))
  }

  fn teardown(&mut self) {}
  fn sync_trace(&mut self, events: Vec<crate::TraceEvent>) {
    for event in events {
      // handle async begin event
      if event.ph == "b" {
        // create a new scope sliced packet if it's not created before
        let (javascript_scoped_descriptor, parent_uuid) = create_scope_sliced_packet(
          event
            .process_name
            .as_deref()
            .unwrap_or(JAVASCRIPT_ANALYSIS_TRACK),
        );
        let mut packet = idl::TracePacket::default();
        // specify the track name for track event using track_descriptor
        let span_track_descriptor =
          create_track_descriptor(unique_uuid(), Some(parent_uuid), event.track_name);
        let final_uuid = span_track_descriptor.uuid();
        let debug_annotations = to_debug_annotation(event.args);
        let mut track_event = create_event(
          final_uuid,
          Some(&event.name),
          None,
          debug_annotations,
          Some(idl::track_event::Type::SliceBegin),
        );
        track_event.categories = event.categories.unwrap_or_default();
        packet.data = Some(idl::trace_packet::Data::TrackEvent(track_event));
        packet.timestamp = Some(event.ts);

        packet.optional_trusted_packet_sequence_id =
          Some(idl::trace_packet::OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(2 as _));
        self.track_state.insert(
          event.uuid,
          (
            span_track_descriptor,
            idl::Trace {
              packet: javascript_scoped_descriptor
                .into_iter()
                .chain(std::iter::once(packet))
                .collect(),
            },
          ),
        );
        // handle async end event
      } else if event.ph == "e" {
        if let Some((desc, mut trace)) = self.track_state.remove(&event.uuid) {
          let mut packet = idl::TracePacket::default();
          let uuid = desc.uuid();
          let debug_annotations = to_debug_annotation(event.args);
          let mut track_event = create_event(
            uuid,
            Some(&event.name),
            None,
            debug_annotations,
            Some(idl::track_event::Type::SliceEnd),
          );
          track_event.categories = event.categories.unwrap_or_default();
          packet.data = Some(idl::trace_packet::Data::TrackEvent(track_event));
          packet.timestamp = Some(event.ts);
          packet.optional_trusted_packet_sequence_id = Some(
            idl::trace_packet::OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(0 as _),
          );
          trace.packet.push(packet);
          self.write_log(&mut trace, Some(desc));
        }
      } else if event.ph == "P" || event.ph == "X" {
        let uuid = self
          .event_id_map
          .entry(event.uuid)
          .or_insert_with(unique_uuid);
        let event_track_descriptor = create_track_descriptor(*uuid, None, event.track_name);
        let final_uuid = event_track_descriptor.uuid();
        let mut packet = idl::TracePacket::default();
        let debug_annotations = to_debug_annotation(event.args);
        let mut track_event = create_event(
          final_uuid,
          Some(&event.name),
          None,
          debug_annotations,
          Some(idl::track_event::Type::Instant),
        );
        track_event.categories = event.categories.unwrap_or_default();
        packet.data = Some(idl::trace_packet::Data::TrackEvent(track_event));
        packet.timestamp = Some(event.ts);

        packet.optional_trusted_packet_sequence_id =
          Some(idl::trace_packet::OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(2 as _));

        let mut trace = idl::Trace {
          packet: vec![packet],
        };
        self.write_log(&mut trace, Some(event_track_descriptor));
      } else {
        // drop not supported events
      }
    }
  }
}

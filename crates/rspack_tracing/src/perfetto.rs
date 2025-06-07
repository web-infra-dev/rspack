use std::{collections::HashMap, io::Write};

use rspack_tracing_perfetto::{
  idl,
  idl::{DebugAnnotationName, TrackDescriptor},
  idl_helpers::{
    create_event, create_track_descriptor, custom_scope_packet, unique_uuid, DebugAnnotations,
  },
  prost::Message,
  BytesMut, PerfettoLayer,
};
static JAVASCRIPT_ANALYSIS_TRACK: &str = "JavaScript Analysis";
use crate::{tracer::Layered, Tracer};
#[derive(Default)]
pub struct PerfettoTracer {
  track_state: HashMap<u32, (TrackDescriptor, idl::Trace)>,
  writer: Option<std::fs::File>,
}
// convert hashmap to perfetto debug annotations
/**
 * is_legacy: whether the annotations are legacy or not
 */
fn to_debug_annotation(
  map: Option<HashMap<String, String>>,
  is_legacy: bool,
) -> (DebugAnnotations, idl::InternedData) {
  let mut interned_data = idl::InternedData::default();
  let mut interned_id = 10086;
  let mut annotations = DebugAnnotations::default();
  if let Some(map) = map {
    for (key, value) in map {
      let mut annotation = idl::DebugAnnotation::default();
      // use iid for legacy annotations
      if is_legacy {
        annotation.name_field = Some(idl::debug_annotation::NameField::NameIid(interned_id));
        interned_data
          .debug_annotation_names
          .push(DebugAnnotationName {
            iid: Some(interned_id),
            name: key.into(),
          });
        interned_id += 1;
      } else {
        annotation.name_field = Some(idl::debug_annotation::NameField::Name(key.into()));
      }

      if is_legacy {
        annotation.value = Some(idl::debug_annotation::Value::LegacyJsonValue(value));
      } else {
        annotation.value = Some(idl::debug_annotation::Value::StringValue(value));
      }

      annotations.annotations.push(annotation);
    }
  }
  (annotations, interned_data)
}
impl PerfettoTracer {
  fn write_log(&mut self, log: &mut idl::Trace, track_descriptor: Option<TrackDescriptor>) {
    let mut buf = BytesMut::new();
    if let Some(task_descriptor) = track_descriptor {
      let mut packet = idl::TracePacket::default();
      packet.data = Some(idl::trace_packet::Data::TrackDescriptor(task_descriptor));
      log.packet.insert(0, packet);
    }

    let Ok(_) = log.encode(&mut buf) else {
      return;
    };
    _ = self.writer.as_ref().unwrap().write_all(&buf);
  }
}
impl Tracer for PerfettoTracer {
  fn setup(&mut self, output: &str) -> Option<Layered> {
    let trace_file = std::fs::File::create(output).unwrap();
    self.writer = trace_file.try_clone().ok();
    let layer = PerfettoLayer::new(trace_file).with_debug_annotations(true);
    Some(Box::new(layer))
  }

  fn teardown(&mut self) {}
  // FIXME: we may use async-local storage to support nested async stack in the future
  fn sync_trace(&mut self, events: Vec<crate::TraceEvent>) {
    for event in events {
      // handle begin
      if event.ph == "b" {
        let (javascript_scoped_descriptor, parent_uuid) =
          custom_scope_packet(JAVASCRIPT_ANALYSIS_TRACK.to_string());
        let mut packet = idl::TracePacket::default();
        let span_track_descriptor = create_track_descriptor(
          unique_uuid(),
          Some(parent_uuid),
          event.track_name,
          None,
          None,
          None,
        );
        let final_uuid = span_track_descriptor.uuid();
        let (debug_annotations, _) = to_debug_annotation(event.args, false);
        let mut track_event = create_event(
          final_uuid,
          Some(&event.name),
          None,
          debug_annotations,
          Some(idl::track_event::Type::SliceBegin),
        );
        track_event.categories = event.category.into_iter().collect();
        packet.data = Some(idl::trace_packet::Data::TrackEvent(track_event));
        packet.timestamp = Some(event.ts as u64);

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
      } else if event.ph == "e" {
        if let Some((desc, mut trace)) = self.track_state.remove(&event.uuid) {
          let mut packet = idl::TracePacket::default();
          let uuid = desc.uuid();
          let (debug_annotations, _) = to_debug_annotation(event.args, false);
          let mut track_event = create_event(
            uuid,
            Some(&event.name),
            None,
            debug_annotations,
            Some(idl::track_event::Type::SliceEnd),
          );
          track_event.categories = event.category.into_iter().collect();
          packet.data = Some(idl::trace_packet::Data::TrackEvent(track_event));
          packet.timestamp = Some(event.ts as u64);
          packet.optional_trusted_packet_sequence_id = Some(
            idl::trace_packet::OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(0 as _),
          );
          trace.packet.push(packet);
          self.write_log(&mut trace, Some(desc));
        } else {
          continue;
        }
      } else if event.ph == "P" {
        let (process_packet, parent_uuid) = custom_scope_packet(
          event
            .process_name
            .unwrap_or(JAVASCRIPT_ANALYSIS_TRACK.to_string()),
        );
        let event_track_descriptor =
          create_track_descriptor(unique_uuid(), None, event.track_name, None, None, None);
        let final_uuid = event_track_descriptor.uuid();
        // handle javascript profiler
        let mut packet = idl::TracePacket::default();
        // P is legacy peretto event
        let (debug_annotations, interned_data) = to_debug_annotation(event.args, true);
        let mut track_event =
          create_event(final_uuid, Some(&event.name), None, debug_annotations, None);
        track_event.categories = event.category.into_iter().collect();
        // This is very tricky and it's reverse engineered from the chrome perfetto trace and source code from
        // https://github.com/google/perfetto/blob/9ddf987d48cdfd9129987a3af1e85052c377756f/src/trace_processor/importers/proto/track_event_tokenizer.cc#L541

        let legacy_event = idl::track_event::LegacyEvent {
          phase: Some(80), // 80 for cpu profiler
          id: Some(idl::track_event::legacy_event::Id::UnscopedId(2)),
          ..Default::default()
        };
        track_event.legacy_event = Some(legacy_event);
        packet.data = Some(idl::trace_packet::Data::TrackEvent(track_event));
        packet.timestamp = Some(event.ts as u64);
        packet.sequence_flags = Some(1);

        packet.optional_trusted_packet_sequence_id =
          Some(idl::trace_packet::OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(2 as _));

        let mut trace = idl::Trace {
          packet: vec![packet],
        };
        for packet in trace.packet.iter_mut() {
          packet.interned_data = Some(interned_data.clone());
        }
        self.write_log(&mut trace, None);
      } else {
        // drop not supported events
      }
    }
  }
}

// Modified base on https://github.com/csmoe/tracing-perfetto
// 1. use micromegas_perfetto to avoid manually updating the perfetto proto file
// 2. use Custom-scoped slices to manage custom scope
#![forbid(unsafe_code)]
use std::io::Write;

pub use bytes::BytesMut;
use idl_helpers::{DebugAnnotations, create_event, create_track_descriptor, unique_uuid};
pub use micromegas_perfetto::protos::{self as idl};
pub use prost;
use prost::Message;
use tracing::{
  Event, Id, Subscriber,
  field::{Field, Visit},
  span,
};
use tracing_subscriber::{Layer, fmt::MakeWriter, layer::Context, registry::LookupSpan};

use crate::idl_helpers::create_scope_sliced_packet;

pub mod idl_helpers;
static DEFAULT_PROCESS_NAME: &str = "Rspack Build Overall";
static DEFAULT_THREAD_NAME: &str = "Main Phase";

pub struct PerfettoSpanState {
  pub track_descriptor: Option<idl::TrackDescriptor>, // optional track descriptor for this span, defaults to thread if not found
  pub trace: idl::Trace, // The Protobuf trace messages that we accumulate for this span.
}

/// A `Layer` that records span as perfetto's
/// `TYPE_SLICE_BEGIN`/`TYPE_SLICE_END`, and event as `TYPE_INSTANT`.
///
/// `PerfettoLayer` will output the records as encoded [protobuf messages](https://github.com/google/perfetto).
pub struct PerfettoLayer<W = fn() -> std::io::Stdout> {
  sequence_id: SequenceId,
  writer: W,
  config: Config,
  start: std::time::Instant,
}

impl<W> PerfettoLayer<W> {
  pub fn get_ts(&self) -> u64 {
    self.start.elapsed().as_nanos() as u64
  }
}

/// Writes encoded records into provided instance.
///
/// This is implemented for types implements [`MakeWriter`].
pub trait PerfettoWriter {
  fn write_log(&self, buf: BytesMut) -> std::io::Result<()>;
  fn flush(&self) -> std::io::Result<()>;
}

impl<W: for<'writer> MakeWriter<'writer> + 'static> PerfettoWriter for W {
  fn write_log(&self, buf: BytesMut) -> std::io::Result<()> {
    self.make_writer().write_all(&buf)
  }
  fn flush(&self) -> std::io::Result<()> {
    Ok(())
  }
}

#[derive(Default)]
struct Config {
  debug_annotations: bool,
  filter: Option<fn(&str) -> bool>,
}

impl<W: PerfettoWriter> PerfettoLayer<W> {
  pub fn new(writer: W) -> Self {
    Self {
      sequence_id: SequenceId::new(0),
      writer,
      config: Config::default(),
      start: std::time::Instant::now(),
    }
  }

  /// Configures whether or not spans/events should be recorded with their metadata and fields.
  pub fn with_debug_annotations(mut self, value: bool) -> Self {
    self.config.debug_annotations = value;
    self
  }
  // flush tracing to disk
  pub fn flush(&self) {
    let _ = self.writer.flush();
  }

  fn write_log(&self, mut log: idl::Trace, track_descriptor: Option<idl::TrackDescriptor>) {
    let mut buf = BytesMut::new();
    let mut packet = idl::TracePacket::default();
    if let Some(track_descriptor) = track_descriptor {
      packet.data = Some(idl::trace_packet::Data::TrackDescriptor(track_descriptor));
    }
    log.packet.insert(1, packet);

    let Ok(_) = log.encode(&mut buf) else {
      return;
    };
    _ = self.writer.write_log(buf);
  }
}

struct SequenceId(u64);

impl SequenceId {
  fn new(n: u64) -> Self {
    Self(n)
  }

  fn get(&self) -> u64 {
    self.0
  }
}

struct TrackNameVisitor<'a> {
  user_track_name: &'a mut Option<String>,
  user_process_name: &'a mut Option<String>,
}

impl<'a> Visit for TrackNameVisitor<'a> {
  // fn record_u64(&mut self, field: &Field, value: u64) {
  //     if field.name() == "perfetto_track_id" {
  //         *self.user_track_id = Some(value);
  //     }
  // }

  fn record_str(&mut self, field: &Field, value: &str) {
    if field.name() == "perfetto.track_name" {
      *self.user_track_name = Some(value.to_string());
    }
    if field.name() == "perfetto.process_name" {
      *self.user_process_name = Some(value.to_string());
    }
  }
  fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {
    // If you want to parse `perfetto_track_id` from a non-u64 typed field,
    // you could do that here, e.g. if user sets `perfetto_track_id = "0xABCD"`.
    // For now, we'll ignore it.
  }
  // Optionally implement record_* for other numeric types if needed
}
struct PerfettoVisitor {
  perfetto: bool,
  filter: fn(&str) -> bool,
}

impl PerfettoVisitor {
  fn new(filter: fn(&str) -> bool) -> PerfettoVisitor {
    Self {
      filter,
      perfetto: false,
    }
  }
}

impl Visit for PerfettoVisitor {
  fn record_debug(&mut self, field: &Field, _value: &dyn std::fmt::Debug) {
    if (self.filter)(field.name()) {
      self.perfetto = true;
    }
  }
}

impl<W, S: Subscriber> Layer<S> for PerfettoLayer<W>
where
  S: for<'a> LookupSpan<'a>,
  W: for<'writer> MakeWriter<'writer> + 'static,
{
  fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
    let Some(span) = ctx.span(id) else {
      return;
    };

    let enabled = self.config.filter.is_none_or(|f| {
      let mut visitor = PerfettoVisitor::new(f);
      attrs.record(&mut visitor);
      visitor.perfetto
    });

    if !enabled {
      return;
    }

    let mut debug_annotations = DebugAnnotations::default();
    if self.config.debug_annotations {
      attrs.record(&mut debug_annotations);
    }

    let mut packet: idl::TracePacket = idl::TracePacket::default();

    // check if parent span has a non default track descriptor
    let inherited_track_descriptor = span
      .parent()
      // If the span has a parent, try retrieving the track descriptor from the parent's state
      .and_then(|parent_span| {
        parent_span
          .extensions()
          .get::<PerfettoSpanState>()
          .map(|state| state.track_descriptor.clone())
      })
      .flatten();

    // retrieve the user set track name and processor name (via `perfetto.track_name` and `perfetto.process_name` field)
    let mut user_track_name = None;
    let mut user_process_name = None;
    let mut visitor = TrackNameVisitor {
      user_track_name: &mut user_track_name,
      user_process_name: &mut user_process_name,
    };

    attrs.record(&mut visitor);
    let (custom_scope_packet, process_uuid) =
      create_scope_sliced_packet(user_process_name.as_deref().unwrap_or(DEFAULT_PROCESS_NAME));

    // resolve the optional track descriptor for this span (either inherited from parent or user set, or None)
    let span_track_descriptor = user_track_name
      .map(|name| {
        create_track_descriptor(
          unique_uuid(),      // uuid
          Some(process_uuid), // use process uuid for separate thread name
          Some(name),         // name
        )
      })
      .or(inherited_track_descriptor)
      .unwrap_or_else(|| {
        create_track_descriptor(
          unique_uuid(),             // uuid
          Some(process_uuid),        // parent_uuid
          Some(DEFAULT_THREAD_NAME), // name
        )
      });

    let final_uuid = span_track_descriptor.uuid();

    let event = create_event(
      final_uuid, // span track id if exists, otherwise thread track id
      Some(span.name()),
      span.metadata().file().zip(span.metadata().line()),
      debug_annotations,
      Some(idl::track_event::Type::SliceBegin),
    );
    packet.data = Some(idl::trace_packet::Data::TrackEvent(event));
    packet.timestamp = Some(self.get_ts());
    packet.trusted_pid = Some(std::process::id() as _);
    packet.optional_trusted_packet_sequence_id = Some(
      idl::trace_packet::OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(
        self.sequence_id.get() as _,
      ),
    );

    let span_state = PerfettoSpanState {
      track_descriptor: Some(span_track_descriptor),
      trace: idl::Trace {
        packet: custom_scope_packet
          .into_iter()
          .chain(std::iter::once(packet))
          .collect(),
      },
    };
    span.extensions_mut().insert(span_state);
  }

  fn on_record(&self, span: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>) {
    let Some(span) = ctx.span(span) else {
      return;
    };

    // We don't check the filter here -- we've already checked it when we handled the span on
    // `on_new_span`. Iff we successfully attached a track packet to the span, then we'll also
    // update the trace packet with the debug data here.
    if let Some(extension) = span.extensions_mut().get_mut::<PerfettoSpanState>()
      && let Some(idl::trace_packet::Data::TrackEvent(event)) = &mut extension.trace.packet[0].data
    {
      let mut debug_annotations = DebugAnnotations::default();
      values.record(&mut debug_annotations);
      event
        .debug_annotations
        .append(&mut debug_annotations.annotations);
    };
  }

  fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
    let enabled = self.config.filter.is_none_or(|f| {
      let mut visitor = PerfettoVisitor::new(f);
      event.record(&mut visitor);
      visitor.perfetto
    });

    if !enabled {
      return;
    }

    let mut user_track_name = None;
    let mut user_process_name = None;
    let mut visitor = TrackNameVisitor {
      user_track_name: &mut user_track_name,
      user_process_name: &mut user_process_name,
    };

    event.record(&mut visitor);
    let inherited_track_descriptor = event
      .parent()
      // If the event has a parent span, try retrieving the track descriptor from the parent's state
      .and_then(|id| {
        let parent_span = ctx.span(id);
        parent_span.and_then(|span| {
          span
            .extensions()
            .get::<PerfettoSpanState>()
            .map(|state| state.track_descriptor.clone())
        })
      })
      .flatten();
    let (custom_scope_packet, process_uuid) =
      create_scope_sliced_packet(user_process_name.as_deref().unwrap_or(DEFAULT_PROCESS_NAME));
    let event_track_descriptor = user_track_name
      .map(|name| {
        create_track_descriptor(
          unique_uuid(),      // uuid
          Some(process_uuid), // use process uuid for separate thread name
          Some(name),         // name
        )
      })
      .or(inherited_track_descriptor)
      .unwrap_or_else(|| {
        create_track_descriptor(
          unique_uuid(),             // uuid
          Some(process_uuid),        // parent_uuid
          Some(DEFAULT_THREAD_NAME), // name
        )
      });
    let metadata = event.metadata();
    let location = metadata.file().zip(metadata.line());

    let mut debug_annotations = DebugAnnotations::default();

    if self.config.debug_annotations {
      event.record(&mut debug_annotations);
    }
    let uuid = unique_uuid();
    let track_event = create_event(
      uuid,
      Some(metadata.name()),
      location,
      debug_annotations,
      Some(idl::track_event::Type::Instant),
    );

    let packet = idl::TracePacket {
      timestamp: Some(self.get_ts()),
      optional_trusted_packet_sequence_id: Some(
        idl::trace_packet::OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(
          self.sequence_id.get() as _,
        ),
      ),
      data: Some(idl::trace_packet::Data::TrackEvent(track_event)),
      ..Default::default()
    };
    let trace = idl::Trace {
      packet: custom_scope_packet
        .into_iter()
        .chain(std::iter::once(packet))
        .collect(),
    };
    self.write_log(trace, Some(event_track_descriptor));
  }

  fn on_close(&self, id: Id, ctx: Context<'_, S>) {
    let Some(span) = ctx.span(&id) else {
      return;
    };

    let Some(mut span_state) = span.extensions_mut().remove::<PerfettoSpanState>() else {
      return;
    };

    let debug_annotations = DebugAnnotations::default();

    let track_uuid = span_state
      .track_descriptor
      .as_ref()
      .map(|d| d.uuid())
      .expect("should have a track descriptor");

    let mut packet = idl::TracePacket::default();
    let meta = span.metadata();
    let event = create_event(
      track_uuid,
      Some(meta.name()),
      meta.file().zip(meta.line()),
      debug_annotations,
      Some(idl::track_event::Type::SliceEnd),
    );
    packet.data = Some(idl::trace_packet::Data::TrackEvent(event));
    packet.timestamp = Some(self.get_ts());
    packet.trusted_pid = Some(std::process::id() as _);
    packet.optional_trusted_packet_sequence_id = Some(
      idl::trace_packet::OptionalTrustedPacketSequenceId::TrustedPacketSequenceId(
        self.sequence_id.get() as _,
      ),
    );

    span_state.trace.packet.push(packet);
    self.write_log(span_state.trace, span_state.track_descriptor);
  }
}

macro_rules! impl_record {
  ($method:ident, $type:ty, $value_variant:ident) => {
    fn $method(&mut self, field: &Field, value: $type) {
      let mut annotation = idl::DebugAnnotation::default();
      annotation.name_field = Some(idl::debug_annotation::NameField::Name(
        field.name().to_string(),
      ));
      annotation.value = Some(idl::debug_annotation::Value::$value_variant(value.into()));
      self.annotations.push(annotation);
    }
  };
  ($method:ident, $type:ty, $value_variant:ident, $conversion:expr) => {
    fn $method(&mut self, field: &Field, value: $type) {
      let mut annotation = idl::DebugAnnotation::default();
      annotation.name_field = Some(idl::debug_annotation::NameField::Name(
        field.name().to_string(),
      ));
      annotation.value = Some(idl::debug_annotation::Value::$value_variant($conversion(
        value,
      )));
      self.annotations.push(annotation);
    }
  };
}

impl Visit for DebugAnnotations {
  impl_record!(record_bool, bool, BoolValue);
  impl_record!(record_str, &str, StringValue, String::from);
  impl_record!(record_f64, f64, DoubleValue);
  impl_record!(record_i64, i64, IntValue);
  impl_record!(record_i128, i128, StringValue, |v: i128| v.to_string());
  impl_record!(record_u128, u128, StringValue, |v: u128| v.to_string());
  impl_record!(record_u64, u64, IntValue, |v: u64| v as i64);

  fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
    let annotation = idl::DebugAnnotation {
      name_field: Some(idl::debug_annotation::NameField::Name(
        field.name().to_string(),
      )),
      value: Some(idl::debug_annotation::Value::StringValue(format!(
        "{value:?}"
      ))),
      ..Default::default()
    };
    self.annotations.push(annotation);
  }

  fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
    let annotation = idl::DebugAnnotation {
      name_field: Some(idl::debug_annotation::NameField::Name(
        field.name().to_string(),
      )),
      value: Some(idl::debug_annotation::Value::StringValue(format!(
        "{value}"
      ))),
      ..Default::default()
    };

    self.annotations.push(annotation);
  }
}

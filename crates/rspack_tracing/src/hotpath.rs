use std::{
  collections::{BTreeMap, HashMap},
  io::Write,
  path::Path,
  sync::{Arc, Mutex},
  time::Instant,
};

use serde::Serialize;
use tracing::{Id, Subscriber, span::Attributes};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};
use unicode_width::UnicodeWidthStr;

use crate::{
  TraceEvent,
  tracer::{Layered, Tracer},
};

#[derive(Default)]
struct SpanMetrics {
  calls: u64,
  total_ns: u64,
  durations_ns: Vec<u64>,
}

impl SpanMetrics {
  fn record(&mut self, duration_ns: u64) {
    self.calls = self.calls.saturating_add(1);
    self.total_ns = self.total_ns.saturating_add(duration_ns);
    self.durations_ns.push(duration_ns);
  }

  fn avg_ns(&self) -> u64 {
    if self.calls == 0 {
      return 0;
    }

    self.total_ns / self.calls
  }

  fn percentile_ns(&self, percentile: u32) -> u64 {
    if self.durations_ns.is_empty() {
      return 0;
    }

    let mut durations = self.durations_ns.clone();
    durations.sort_unstable();
    let len = durations.len();
    let rank = (len.saturating_mul(percentile as usize)).div_ceil(100);
    let index = rank.saturating_sub(1).min(len - 1);
    durations[index]
  }
}

#[derive(Default)]
struct ChartsState {
  metrics: HashMap<String, SpanMetrics>,
  pending_js_events: HashMap<u32, PendingJsEvent>,
}

impl ChartsState {
  fn record_duration(&mut self, key: String, duration_ns: u64) {
    self.metrics.entry(key).or_default().record(duration_ns);
  }

  fn record_js_begin(&mut self, event: TraceEvent) {
    self.pending_js_events.insert(
      event.uuid,
      PendingJsEvent {
        key: format_js_name(&event),
        started_at_ns: event.ts,
      },
    );
  }

  fn record_js_end(&mut self, event: &TraceEvent) {
    if let Some(begin_event) = self.pending_js_events.remove(&event.uuid) {
      let duration_ns = event.ts.saturating_sub(begin_event.started_at_ns);
      self.record_duration(begin_event.key, duration_ns);
    }
  }
}

struct PendingJsEvent {
  key: String,
  started_at_ns: u64,
}

struct SpanTiming {
  key: String,
  started_at: Instant,
}

struct ChartsLayer {
  state: Arc<Mutex<ChartsState>>,
}

const CHARTS_PERCENTILES: [u8; 1] = [95];

impl<S> Layer<S> for ChartsLayer
where
  S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
  fn on_new_span(&self, _attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
    let Some(span) = ctx.span(id) else {
      return;
    };

    let metadata = span.metadata();
    span.extensions_mut().insert(SpanTiming {
      key: format_rust_name(metadata.target(), metadata.name()),
      started_at: Instant::now(),
    });
  }

  fn on_close(&self, id: Id, ctx: Context<'_, S>) {
    let Some(span) = ctx.span(&id) else {
      return;
    };

    let Some(span_timing) = span.extensions_mut().remove::<SpanTiming>() else {
      return;
    };

    let duration_ns =
      u64::try_from(span_timing.started_at.elapsed().as_nanos()).unwrap_or(u64::MAX);
    self
      .state
      .lock()
      .expect("Failed to lock hotpath trace state")
      .record_duration(span_timing.key, duration_ns);
  }
}

pub struct HotpathTracer {
  started_at: Option<Instant>,
  state: Arc<Mutex<ChartsState>>,
  output: Option<ChartsOutput>,
}

impl Default for HotpathTracer {
  fn default() -> Self {
    Self {
      started_at: None,
      state: Arc::new(Mutex::new(ChartsState::default())),
      output: None,
    }
  }
}

impl Tracer for HotpathTracer {
  fn setup(&mut self, output: &str) -> Option<Layered> {
    self.started_at = Some(Instant::now());
    self.output = Some(create_output(output));

    Some(Box::new(ChartsLayer {
      state: self.state.clone(),
    }))
  }

  fn sync_trace(&mut self, events: Vec<TraceEvent>) {
    let mut state = self
      .state
      .lock()
      .expect("Failed to lock hotpath trace state");

    for event in events {
      match event.ph.as_str() {
        "b" => state.record_js_begin(event),
        "e" => state.record_js_end(&event),
        _ => {}
      }
    }
  }

  fn teardown(&mut self) {
    let Some(output) = self.output.as_mut() else {
      return;
    };
    let Some(started_at) = self.started_at else {
      return;
    };

    let total_elapsed_ns = u64::try_from(started_at.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let report = {
      let state = self
        .state
        .lock()
        .expect("Failed to lock hotpath trace state");
      match output.format {
        ChartsOutputFormat::Table => render_table_report(&state, total_elapsed_ns),
        ChartsOutputFormat::Json => render_json_report(&state, total_elapsed_ns),
      }
    };

    let _ = output.writer.write_all(report.as_bytes());
    let _ = output.writer.flush();
  }
}

struct ChartsOutput {
  format: ChartsOutputFormat,
  writer: Box<dyn Write + Send>,
}

#[derive(Clone, Copy)]
enum ChartsOutputFormat {
  Table,
  Json,
}

fn create_output(output: &str) -> ChartsOutput {
  ChartsOutput {
    format: detect_output_format(output),
    writer: match output {
      "stdout" => Box::new(std::io::stdout()),
      "stderr" => Box::new(std::io::stderr()),
      path => {
        let file = std::fs::File::create(path)
          .unwrap_or_else(|e| panic!("Failed to create trace file: {path} due to {e}"));
        Box::new(file)
      }
    },
  }
}

fn detect_output_format(output: &str) -> ChartsOutputFormat {
  match output {
    "stdout" | "stderr" => ChartsOutputFormat::Table,
    path => {
      let is_json = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"));

      if is_json {
        ChartsOutputFormat::Json
      } else {
        ChartsOutputFormat::Table
      }
    }
  }
}

fn format_rust_name(target: &str, name: &str) -> String {
  if name == target || name.starts_with(target) {
    return name.to_string();
  }

  format!("{target}::{name}")
}

fn format_js_name(event: &TraceEvent) -> String {
  let mut parts = vec!["javascript".to_string()];

  for part in [
    event.process_name.as_deref(),
    event.track_name.as_deref(),
    Some(event.name.as_str()),
  ]
  .into_iter()
  .flatten()
  {
    if parts.last().is_none_or(|last| last != part) {
      parts.push(part.to_string());
    }
  }

  parts.join("::")
}

fn shorten_name(name: &str) -> String {
  let parts: Vec<&str> = name.split("::").collect();
  if parts.len() > 2 {
    return parts[parts.len() - 2..].join("::");
  }

  name.to_string()
}

fn format_duration_ns(ns: u64) -> String {
  if ns < 1_000 {
    format!("{ns} ns")
  } else if ns < 1_000_000 {
    format!("{:.2} us", ns as f64 / 1_000.0)
  } else if ns < 1_000_000_000 {
    format!("{:.2} ms", ns as f64 / 1_000_000.0)
  } else {
    format!("{:.2} s", ns as f64 / 1_000_000_000.0)
  }
}

fn calculate_percentage_basis_points(total_ns: u64, total_elapsed_ns: u64) -> u64 {
  if total_elapsed_ns == 0 {
    return 0;
  }

  ((total_ns as f64 / total_elapsed_ns as f64) * 10_000.0).round() as u64
}

fn format_percentage_from_basis_points(basis_points: u64) -> String {
  format!("{:.2}%", basis_points as f64 / 100.0)
}

fn collect_entries(state: &ChartsState, total_elapsed_ns: u64) -> Vec<ChartsEntry> {
  let mut entries = state
    .metrics
    .iter()
    .map(|(name, metrics)| ChartsEntry {
      name: name.clone(),
      short_name: shorten_name(name),
      calls: metrics.calls,
      avg_ns: metrics.avg_ns(),
      p95_ns: metrics.percentile_ns(CHARTS_PERCENTILES[0].into()),
      total_ns: metrics.total_ns,
      percent_total_basis_points: calculate_percentage_basis_points(
        metrics.total_ns,
        total_elapsed_ns,
      ),
    })
    .collect::<Vec<_>>();

  entries.sort_by(|left, right| {
    right
      .total_ns
      .cmp(&left.total_ns)
      .then_with(|| left.name.cmp(&right.name))
  });

  entries
}

fn render_table_report(state: &ChartsState, total_elapsed_ns: u64) -> String {
  let entries = collect_entries(state, total_elapsed_ns);

  if entries.is_empty() {
    return format!(
      "hotpath - Aggregated tracing spans (avg, p95, total)\nOverall elapsed: {}\nNo tracing spans were captured.\n",
      format_duration_ns(total_elapsed_ns)
    );
  }

  let table_rows = entries
    .into_iter()
    .map(|entry| {
      vec![
        entry.short_name,
        entry.calls.to_string(),
        format_duration_ns(entry.avg_ns),
        format_duration_ns(entry.p95_ns),
        format_duration_ns(entry.total_ns),
        format_percentage_from_basis_points(entry.percent_total_basis_points),
      ]
    })
    .collect::<Vec<_>>();
  let headers = vec![
    "Function".to_string(),
    "Calls".to_string(),
    "Avg".to_string(),
    "P95".to_string(),
    "Total".to_string(),
    "% Total".to_string(),
  ];

  let mut output = String::new();
  output.push_str("hotpath - Aggregated tracing spans (avg, p95, total)\n");
  output.push_str(&format!(
    "Overall elapsed: {}\n",
    format_duration_ns(total_elapsed_ns)
  ));
  output.push_str(&render_table(&headers, &table_rows));
  output.push('\n');
  output
}

#[derive(Clone)]
struct ChartsEntry {
  name: String,
  short_name: String,
  calls: u64,
  avg_ns: u64,
  p95_ns: u64,
  total_ns: u64,
  percent_total_basis_points: u64,
}

#[derive(Serialize)]
struct JsonChartsReport {
  rspack_trace_layer: &'static str,
  output_format: &'static str,
  description: &'static str,
  time_elapsed: String,
  total_elapsed_ns: u64,
  percentiles: Vec<u8>,
  data: Vec<JsonChartsEntry>,
}

#[derive(Serialize)]
struct JsonChartsEntry {
  name: String,
  calls: u64,
  avg: String,
  avg_raw: u64,
  #[serde(flatten)]
  percentiles: BTreeMap<String, String>,
  percentiles_raw: BTreeMap<String, u64>,
  total: String,
  total_raw: u64,
  percent_total: String,
  percent_total_raw: u64,
}

fn render_json_report(state: &ChartsState, total_elapsed_ns: u64) -> String {
  let entries = collect_entries(state, total_elapsed_ns);
  let report = JsonChartsReport {
    rspack_trace_layer: "hotpath",
    output_format: "json",
    description: "Aggregated tracing spans (avg, p95, total)",
    time_elapsed: format_duration_ns(total_elapsed_ns),
    total_elapsed_ns,
    percentiles: CHARTS_PERCENTILES.to_vec(),
    data: entries
      .into_iter()
      .map(|entry| JsonChartsEntry {
        name: entry.name,
        calls: entry.calls,
        avg: format_duration_ns(entry.avg_ns),
        avg_raw: entry.avg_ns,
        percentiles: BTreeMap::from([("p95".to_string(), format_duration_ns(entry.p95_ns))]),
        percentiles_raw: BTreeMap::from([("p95".to_string(), entry.p95_ns)]),
        total: format_duration_ns(entry.total_ns),
        total_raw: entry.total_ns,
        percent_total: format_percentage_from_basis_points(entry.percent_total_basis_points),
        percent_total_raw: entry.percent_total_basis_points,
      })
      .collect(),
  };

  let mut output =
    serde_json::to_string_pretty(&report).expect("Failed to serialize hotpath trace report");
  output.push('\n');
  output
}

fn render_table(headers: &[String], rows: &[Vec<String>]) -> String {
  let mut widths = headers
    .iter()
    .map(|header| UnicodeWidthStr::width(header.as_str()))
    .collect::<Vec<_>>();

  for row in rows {
    for (index, cell) in row.iter().enumerate() {
      widths[index] = widths[index].max(UnicodeWidthStr::width(cell.as_str()));
    }
  }

  let mut output = String::new();
  output.push_str(&render_separator(&widths));
  output.push_str(&render_row(headers, &widths, false));
  output.push_str(&render_separator(&widths));

  for row in rows {
    output.push_str(&render_row(row, &widths, true));
  }

  output.push_str(&render_separator(&widths));
  output
}

fn render_separator(widths: &[usize]) -> String {
  let mut output = String::new();
  output.push('+');
  for width in widths {
    output.push_str(&"-".repeat(width.saturating_add(2)));
    output.push('+');
  }
  output.push('\n');
  output
}

fn render_row(cells: &[String], widths: &[usize], align_numbers_right: bool) -> String {
  let mut output = String::new();
  output.push('|');

  for (index, cell) in cells.iter().enumerate() {
    output.push(' ');

    let width = widths[index];
    let cell_width = UnicodeWidthStr::width(cell.as_str());
    let padding = width.saturating_sub(cell_width);
    let align_right = align_numbers_right && index > 0;

    if align_right {
      output.push_str(&" ".repeat(padding));
      output.push_str(cell);
    } else {
      output.push_str(cell);
      output.push_str(&" ".repeat(padding));
    }

    output.push(' ');
    output.push('|');
  }

  output.push('\n');
  output
}

#[cfg(test)]
mod tests {
  use serde_json::Value;

  use super::{ChartsState, TraceEvent, render_json_report, render_table_report};

  #[test]
  fn should_render_hotpath_like_table() {
    let mut state = ChartsState::default();
    state.record_duration("rspack_core::compiler::make".to_string(), 2_000_000);
    state.record_duration("rspack_core::compiler::make".to_string(), 4_000_000);
    state.record_duration("rspack_core::compiler::seal".to_string(), 1_000_000);

    let output = render_table_report(&state, 10_000_000);

    assert!(output.contains("hotpath - Aggregated tracing spans"));
    assert!(output.contains("Overall elapsed: 10.00 ms"));
    assert!(output.contains("| Function"));
    assert!(output.contains("| compiler::make"));
    assert!(output.contains("3.00 ms"));
    assert!(output.contains("4.00 ms"));
    assert!(output.contains("6.00 ms"));
    assert!(output.contains("60.00%"));
  }

  #[test]
  fn should_aggregate_javascript_async_events() {
    let mut state = ChartsState::default();
    state.record_js_begin(TraceEvent {
      name: "make".to_string(),
      track_name: Some("HtmlRspackPlugin".to_string()),
      process_name: Some("Plugin Analysis".to_string()),
      args: None,
      uuid: 1,
      ts: 100,
      ph: "b".to_string(),
      categories: None,
    });
    state.record_js_end(&TraceEvent {
      name: "make".to_string(),
      track_name: Some("HtmlRspackPlugin".to_string()),
      process_name: Some("Plugin Analysis".to_string()),
      args: None,
      uuid: 1,
      ts: 2_100,
      ph: "e".to_string(),
      categories: None,
    });

    let output = render_table_report(&state, 10_000);

    assert!(output.contains("Overall elapsed: 10.00 us"));
    assert!(output.contains("HtmlRspackPlugin::make"));
    assert!(output.contains("2.00 us"));
  }

  #[test]
  fn should_render_hotpath_like_json() {
    let mut state = ChartsState::default();
    state.record_duration("rspack_core::compiler::make".to_string(), 2_000_000);
    state.record_duration("rspack_core::compiler::make".to_string(), 4_000_000);

    let output = render_json_report(&state, 10_000_000);
    let json: Value = serde_json::from_str(&output).expect("Expected valid JSON output");

    assert_eq!(json["rspack_trace_layer"], "hotpath");
    assert_eq!(json["output_format"], "json");
    assert_eq!(json["time_elapsed"], "10.00 ms");
    assert_eq!(json["percentiles"], serde_json::json!([95]));
    assert_eq!(json["data"][0]["name"], "rspack_core::compiler::make");
    assert_eq!(json["data"][0]["avg_raw"], 3_000_000);
    assert_eq!(json["data"][0]["percentiles_raw"]["p95"], 4_000_000);
    assert_eq!(json["data"][0]["total_raw"], 6_000_000);
    assert_eq!(json["data"][0]["percent_total_raw"], 6000);
  }
}

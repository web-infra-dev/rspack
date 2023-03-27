use rspack_core::{BoxPlugin, ExternalItem, PluginExt};
use rspack_regex::RspackRegex;

pub fn node_target_plugin() -> BoxPlugin {
  crate::ExternalPlugin::new(
    "commonjs".to_string(), // TODO: should be "node-commonjs"
    vec![
      ExternalItem::from("assert".to_string()),
      ExternalItem::from("assert/strict".to_string()),
      ExternalItem::from("async_hooks".to_string()),
      ExternalItem::from("buffer".to_string()),
      ExternalItem::from("child_process".to_string()),
      ExternalItem::from("cluster".to_string()),
      ExternalItem::from("console".to_string()),
      ExternalItem::from("constants".to_string()),
      ExternalItem::from("crypto".to_string()),
      ExternalItem::from("dgram".to_string()),
      ExternalItem::from("diagnostics_channel".to_string()),
      ExternalItem::from("dns".to_string()),
      ExternalItem::from("dns/promises".to_string()),
      ExternalItem::from("domain".to_string()),
      ExternalItem::from("events".to_string()),
      ExternalItem::from("fs".to_string()),
      ExternalItem::from("fs/promises".to_string()),
      ExternalItem::from("http".to_string()),
      ExternalItem::from("http2".to_string()),
      ExternalItem::from("https".to_string()),
      ExternalItem::from("inspector".to_string()),
      ExternalItem::from("inspector/promises".to_string()),
      ExternalItem::from("module".to_string()),
      ExternalItem::from("net".to_string()),
      ExternalItem::from("os".to_string()),
      ExternalItem::from("path".to_string()),
      ExternalItem::from("path/posix".to_string()),
      ExternalItem::from("path/win32".to_string()),
      ExternalItem::from("perf_hooks".to_string()),
      ExternalItem::from("process".to_string()),
      ExternalItem::from("punycode".to_string()),
      ExternalItem::from("querystring".to_string()),
      ExternalItem::from("readline".to_string()),
      ExternalItem::from("readline/promises".to_string()),
      ExternalItem::from("repl".to_string()),
      ExternalItem::from("stream".to_string()),
      ExternalItem::from("stream/consumers".to_string()),
      ExternalItem::from("stream/promises".to_string()),
      ExternalItem::from("stream/web".to_string()),
      ExternalItem::from("string_decoder".to_string()),
      ExternalItem::from("sys".to_string()),
      ExternalItem::from("timers".to_string()),
      ExternalItem::from("timers/promises".to_string()),
      ExternalItem::from("tls".to_string()),
      ExternalItem::from("trace_events".to_string()),
      ExternalItem::from("tty".to_string()),
      ExternalItem::from("url".to_string()),
      ExternalItem::from("util".to_string()),
      ExternalItem::from("util/types".to_string()),
      ExternalItem::from("v8".to_string()),
      ExternalItem::from("vm".to_string()),
      ExternalItem::from("wasi".to_string()),
      ExternalItem::from("worker_threads".to_string()),
      ExternalItem::from("zlib".to_string()),
      ExternalItem::from(RspackRegex::new("^node:").expect("Invalid regexp")),
      // Yarn PnP adds pnpapi as "builtin"
      ExternalItem::from("pnpapi".to_string()),
    ],
  )
  .boxed()
}

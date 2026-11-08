use std::path::Path;

use crate::{ResolveError, ResolveOptions, Resolver};

#[tokio::test]
async fn builtins_off() {
  let f = Path::new("/");
  let resolver = Resolver::default();
  let resolved_path = resolver.resolve(f, "zlib").await.map(|r| r.full_path());
  assert_eq!(resolved_path, Err(ResolveError::NotFound("zlib".into())));
}

#[tokio::test]
async fn builtins() {
  let f = Path::new("/");

  let resolver = Resolver::new(ResolveOptions::default().with_builtin_modules(true));

  let pass = [
    "_http_agent",
    "_http_client",
    "_http_common",
    "_http_incoming",
    "_http_outgoing",
    "_http_server",
    "_stream_duplex",
    "_stream_passthrough",
    "_stream_readable",
    "_stream_transform",
    "_stream_wrap",
    "_stream_writable",
    "_tls_common",
    "_tls_wrap",
    "assert",
    "assert/strict",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "dns/promises",
    "domain",
    "events",
    "fs",
    "fs/promises",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "path/posix",
    "path/win32",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "stream/consumers",
    "stream/promises",
    "stream/web",
    "string_decoder",
    "sys",
    "timers",
    "timers/promises",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "util/types",
    "v8",
    "vm",
    "worker_threads",
    "zlib",
  ];

  for request in pass {
    let prefixed_request = format!("node:{request}");
    for request in [prefixed_request.clone(), request.to_string()] {
      let resolved_path = resolver.resolve(f, &request).await.map(|r| r.full_path());
      let err = ResolveError::Builtin(prefixed_request.clone());
      assert_eq!(resolved_path, Err(err), "{request}");
    }
  }
}

#[tokio::test]
async fn fail() {
  let f = Path::new("/");
  let resolver = Resolver::new(ResolveOptions::default().with_builtin_modules(true));
  let request = "xxx";
  let resolved_path = resolver.resolve(f, request).await;
  let err = ResolveError::NotFound(request.to_string());
  assert_eq!(resolved_path, Err(err), "{request}");
}

#[tokio::test]
async fn imports() {
  let f = super::fixture().join("builtins");
  let resolver = Resolver::new(ResolveOptions {
    builtin_modules: true,
    condition_names: vec!["node".into()],
    ..ResolveOptions::default()
  });

  for request in ["#fs", "#http"] {
    let resolved_path = resolver
      .resolve(f.clone(), request)
      .await
      .map(|r| r.full_path());
    let err = ResolveError::Builtin(format!("node:{}", request.trim_start_matches('#')));
    assert_eq!(resolved_path, Err(err));
  }
}

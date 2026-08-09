#[cfg(not(unix))]
compile_error!("this reproduction currently supports macOS and Linux");

use std::{
  env, fs,
  hint::black_box,
  mem::MaybeUninit,
  path::{Path, PathBuf},
  process,
};

struct Fixture(PathBuf);

impl Fixture {
  fn new(version: &str, reference_count: usize) -> Self {
    let root = env::temp_dir().join(format!(
      "rspack-issue-15021-{}-{version}-{reference_count}",
      process::id()
    ));
    fs::create_dir_all(root.join("app")).expect("create fixture app directory");
    fs::write(root.join("app/target.js"), "export default 1;\n").expect("write fixture target");

    let mut references = String::new();
    for index in 0..reference_count {
      let project = format!("project-{index:05}");
      fs::create_dir_all(root.join(&project)).expect("create referenced project");
      fs::write(
        root.join(&project).join("tsconfig.json"),
        r#"{"compilerOptions":{"composite":true}}"#,
      )
      .expect("write referenced tsconfig");
      if index != 0 {
        references.push(',');
      }
      references.push_str(&format!(r#"{{"path":"../{project}"}}"#));
    }

    let root_config = format!(
      r#"{{
        "compilerOptions": {{
          "baseUrl": ".",
          "paths": {{ "target": ["./target.js"] }}
        }},
        "references": [{references}]
      }}"#
    );
    fs::write(root.join("app/tsconfig.json"), root_config).expect("write root tsconfig");
    Self(root)
  }

  fn root(&self) -> &Path {
    &self.0
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn peak_rss_mib() -> f64 {
  let mut usage = MaybeUninit::<libc::rusage>::uninit();
  let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
  assert_eq!(result, 0, "getrusage failed");
  let raw = unsafe { usage.assume_init().ru_maxrss } as f64;

  #[cfg(target_os = "macos")]
  let bytes = raw;
  #[cfg(not(target_os = "macos"))]
  let bytes = raw * 1024.0;

  bytes / (1024.0 * 1024.0)
}

async fn run_08(root: &Path, iterations: usize) {
  use resolver08::{ResolveContext, ResolveOptions, Resolver, TsconfigOptions, TsconfigReferences};

  let app = root.join("app");
  let resolver = Resolver::new(ResolveOptions {
    tsconfig: Some(TsconfigOptions {
      config_file: app.join("tsconfig.json"),
      references: TsconfigReferences::Auto,
    }),
    extensions: vec![".js".into()],
    ..ResolveOptions::default()
  });
  let mut retained = Vec::with_capacity(iterations);
  for _ in 0..iterations {
    let mut context = ResolveContext::default();
    resolver
      .resolve_with_context(&app, "target", &mut context)
      .await
      .expect("resolver 0.8 should resolve target");
    retained.push(context);
  }
  let per_context = retained
    .first()
    .map(|context| context.file_dependencies.len() + context.missing_dependencies.len())
    .unwrap_or_default();
  let total_entries: usize = retained
    .iter()
    .map(|context| context.file_dependencies.len() + context.missing_dependencies.len())
    .sum();
  println!(
    "resolver=0.8 contexts={iterations} entries_per_context={per_context} total_retained_entries={total_entries} peak_rss_mib={:.2}",
    peak_rss_mib()
  );
  black_box(&retained);
}

async fn run_09(root: &Path, iterations: usize) {
  use resolver09::{ResolveContext, ResolveOptions, Resolver, TsconfigOptions, TsconfigReferences};

  let app = root.join("app");
  let resolver = Resolver::new(ResolveOptions {
    tsconfig: Some(TsconfigOptions {
      config_file: app.join("tsconfig.json"),
      references: TsconfigReferences::Auto,
    }),
    extensions: vec![".js".into()],
    ..ResolveOptions::default()
  });
  let mut retained = Vec::with_capacity(iterations);
  for _ in 0..iterations {
    let mut context = ResolveContext::default();
    resolver
      .resolve_with_context(&app, "target", &mut context)
      .await
      .expect("resolver 0.9 should resolve target");
    retained.push(context);
  }
  let per_context = retained
    .first()
    .map(|context| context.file_dependencies.len() + context.missing_dependencies.len())
    .unwrap_or_default();
  let total_entries: usize = retained
    .iter()
    .map(|context| context.file_dependencies.len() + context.missing_dependencies.len())
    .sum();
  println!(
    "resolver=0.9 contexts={iterations} entries_per_context={per_context} total_retained_entries={total_entries} peak_rss_mib={:.2}",
    peak_rss_mib()
  );
  black_box(&retained);
}

#[tokio::main]
async fn main() {
  let arguments = env::args().collect::<Vec<_>>();
  let version = arguments.get(1).map(String::as_str).unwrap_or("0.9");
  let reference_count = arguments
    .get(2)
    .and_then(|value| value.parse().ok())
    .unwrap_or(300);
  let iterations = arguments
    .get(3)
    .and_then(|value| value.parse().ok())
    .unwrap_or(1000);
  let fixture = Fixture::new(version, reference_count);

  match version {
    "0.8" => run_08(fixture.root(), iterations).await,
    "0.9" => run_09(fixture.root(), iterations).await,
    other => panic!("unsupported resolver version {other}; expected 0.8 or 0.9"),
  }
}

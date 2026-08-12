#[cfg(target_family = "wasm")]
use std::alloc::System;
use std::{
  alloc::{GlobalAlloc, Layout},
  env, fs,
  fs::read_to_string,
  future::Future,
  io::{self, Write},
  path::{Path, PathBuf},
  sync::Arc,
};

#[global_allocator]
#[cfg(not(target_family = "wasm"))]
static GLOBAL: NeverGrowInPlaceAllocator<mimalloc::MiMalloc> =
  NeverGrowInPlaceAllocator::new(mimalloc::MiMalloc);

#[global_allocator]
#[cfg(target_family = "wasm")]
static GLOBAL: NeverGrowInPlaceAllocator<System> = NeverGrowInPlaceAllocator::new(System);

/// Delegates `alloc`/`dealloc` to the wrapped allocator but omits
/// [`GlobalAlloc::realloc`], forcing the default "alloc-new + copy + dealloc-old"
/// path so that benchmarks never benefit from non-deterministic in-place growth
/// provided by the underlying allocator's `realloc`. Wrapping `mimalloc::MiMalloc`
/// (instead of using it directly) also keeps `alloc` / `dealloc` visible to
/// CodSpeed's mimalloc white-box allocator tracking.
struct NeverGrowInPlaceAllocator<A> {
  allocator: A,
}

impl<A> NeverGrowInPlaceAllocator<A> {
  const fn new(allocator: A) -> Self {
    Self { allocator }
  }
}

// SAFETY: Methods simply delegate to the wrapped allocator.
unsafe impl<A: GlobalAlloc> GlobalAlloc for NeverGrowInPlaceAllocator<A> {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    self.allocator.alloc(layout)
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    self.allocator.dealloc(ptr, layout)
  }
}

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rspack_resolver::{FileSystemOptions, FileSystemOs, ResolveOptions, Resolver};
use serde_json::Value;
use tokio::{
  runtime::{self, Builder},
  task::JoinSet,
};

fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
  #[cfg(target_family = "unix")]
  {
    std::os::unix::fs::symlink(original, link)
  }

  #[cfg(target_family = "windows")]
  {
    std::os::windows::fs::symlink_file(original, link)
  }
}

fn create_symlinks() -> io::Result<PathBuf> {
  let root = env::current_dir()?.join("fixtures/enhanced_resolve");
  let dirname = root.join("test");
  let temp_path = dirname.join("temp_symlinks");
  let create_symlink_fixtures = || -> io::Result<()> {
    fs::create_dir(&temp_path)?;
    let mut index = fs::File::create(temp_path.join("index.js"))?;
    index.write_all(b"console.log('Hello, World!')")?;
    // create 10000 symlink files pointing to the index.js
    for i in 0..10000 {
      symlink(
        temp_path.join("index.js"),
        temp_path.join(format!("file{i}.js")),
      )?;
    }
    Ok(())
  };
  if !temp_path.exists() {
    if let Err(err) = create_symlink_fixtures() {
      let _ = fs::remove_dir_all(&temp_path);
      return Err(err);
    }
  }
  Ok(temp_path)
}

fn rspack_resolver(enable_pnp: bool) -> rspack_resolver::Resolver {
  use rspack_resolver::{AliasValue, ResolveOptions, Resolver};
  let alias_value = AliasValue::from("./");

  let fs = FileSystemOs::new(FileSystemOptions {
    #[cfg(feature = "yarn_pnp")]
    enable_pnp,
  });

  Resolver::new_with_file_system(
    fs,
    ResolveOptions {
      #[cfg(feature = "yarn_pnp")]
      enable_pnp,
      extensions: vec![".ts".into(), ".js".into(), ".mjs".into()],
      condition_names: vec!["import".into(), "webpack".into(), "require".into()],
      alias_fields: vec![vec!["browser".into()]],
      extension_alias: vec![(".js".into(), vec![".ts".into(), ".js".into()])],
      // Real projects LOVE setting these many aliases.
      // I saw them with my own eyes.
      alias: vec![
        ("/absolute/path".into(), vec![alias_value.clone()]),
        ("aaa".into(), vec![alias_value.clone()]),
        ("bbb".into(), vec![alias_value.clone()]),
        ("ccc".into(), vec![alias_value.clone()]),
        ("ddd".into(), vec![alias_value.clone()]),
        ("eee".into(), vec![alias_value.clone()]),
        ("fff".into(), vec![alias_value.clone()]),
        ("ggg".into(), vec![alias_value.clone()]),
        ("hhh".into(), vec![alias_value.clone()]),
        ("iii".into(), vec![alias_value.clone()]),
        ("jjj".into(), vec![alias_value.clone()]),
        ("kkk".into(), vec![alias_value.clone()]),
        ("lll".into(), vec![alias_value.clone()]),
        ("mmm".into(), vec![alias_value.clone()]),
        ("nnn".into(), vec![alias_value.clone()]),
        ("ooo".into(), vec![alias_value.clone()]),
        ("ppp".into(), vec![alias_value.clone()]),
        ("qqq".into(), vec![alias_value.clone()]),
        ("rrr".into(), vec![alias_value.clone()]),
        ("sss".into(), vec![alias_value.clone()]),
        ("@".into(), vec![alias_value.clone()]),
        ("@@".into(), vec![alias_value.clone()]),
        ("@@@".into(), vec![alias_value]),
      ],
      ..ResolveOptions::default()
    },
  )
}

fn resolver_with_many_extensions() -> rspack_resolver::Resolver {
  Resolver::new(ResolveOptions {
    extensions: vec![
      ".bad0".to_string(),
      ".bad1".to_string(),
      ".bad2".to_string(),
      ".bad3".to_string(),
      ".bad4".to_string(),
      ".bad5".to_string(),
      ".bad6".to_string(),
      ".bad7".to_string(),
      ".bad8".to_string(),
      ".bad9".to_string(),
      ".mtsx".to_string(),
      ".mts".to_string(),
      ".mjs".to_string(),
      ".tsx".to_string(),
      ".ts".to_string(),
      ".jsx".to_string(),
      ".js".to_string(),
    ],
    imports_fields: vec![],
    exports_fields: vec![],
    enable_pnp: false,
    ..Default::default()
  })
}

fn tsconfig_resolver() -> rspack_resolver::Resolver {
  use rspack_resolver::{ResolveOptions, Resolver, TsconfigOptions, TsconfigReferences};
  let config_dir = env::current_dir()
    .unwrap()
    .join("fixtures/tsconfig/cases/project_references");
  Resolver::new(ResolveOptions {
    extensions: vec![".ts".into(), ".js".into()],
    tsconfig: Some(TsconfigOptions {
      config_file: config_dir.join("app"),
      references: TsconfigReferences::Auto,
    }),
    ..ResolveOptions::default()
  })
}

fn create_async_resolve_task(
  rspack_resolver: Arc<rspack_resolver::Resolver>,
  path: PathBuf,
  request: String,
) -> impl Future<Output = ()> {
  async move {
    let _ = rspack_resolver.resolve(path, &request).await;
  }
}

fn bench_resolver(c: &mut Criterion) {
  let cwd = env::current_dir().unwrap().join("benches");

  let pkg_content = read_to_string("./benches/package.json").unwrap();
  let pkg_json: Value = serde_json::from_str(&pkg_content).unwrap();
  // about 1000 npm packages
  let data = pkg_json["dependencies"]
    .as_object()
    .unwrap()
    .keys()
    .map(|name| (&cwd, name))
    .collect::<Vec<_>>();

  // check validity
  runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
        for (path, request) in &data {
            let r = rspack_resolver(false).resolve(path, request).await;
            if !r.is_ok() {
                panic!("resolve failed {path:?} {request},\n\nplease run `pnpm install` in `/benches` before running the benchmarks");
            }
        }
    });

  let symlink_test_dir = create_symlinks().expect("Create symlink fixtures failed");

  let symlinks_range = 0u32..10000;

  // check validity
  runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
      for i in symlinks_range.clone() {
        assert!(
          rspack_resolver(false)
            .resolve(&symlink_test_dir, &format!("./file{i}"))
            .await
            .is_ok(),
          "file{i}.js"
        );
      }
    });

  let mut group = c.benchmark_group("resolver");

  // CodSpeed memory mode (Valgrind massif) cannot meaningfully measure
  // multi-threaded throughput — heap snapshots conflate allocations across
  // worker threads and produce noisy, non-actionable deltas.
  let skip_threaded = env::var("CODSPEED_RUNNER_MODE").as_deref() == Ok("memory");

  // codspeed can only handle to up to 500 threads
  let multi_rt = || {
    Builder::new_multi_thread()
      .max_blocking_threads(256)
      .build()
      .expect("failed to create tokio runtime")
  };

  // force to use four threads
  rayon::ThreadPoolBuilder::new()
    .num_threads(4)
    .build_global()
    .expect("Failed to build global thread pool");

  group.bench_with_input(
    BenchmarkId::from_parameter("single-thread"),
    &data,
    |b, data| {
      let runner = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

      let rspack_resolver = rspack_resolver(false);

      b.to_async(runner).iter_with_setup(
        || {
          rspack_resolver.clear_cache();
        },
        |_| async {
          for (path, request) in data {
            _ = rspack_resolver.resolve(path, request).await;
          }
        },
      );
    },
  );

  group.bench_with_input(
    BenchmarkId::from_parameter("[single-threaded]resolve with many extensions"),
    &data,
    |b, data| {
      let runner = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");
      let rspack_resolver = resolver_with_many_extensions();

      b.to_async(runner).iter_with_setup(
        || {
          rspack_resolver.clear_cache();
        },
        |_| async {
          for (path, request) in data {
            _ = rspack_resolver
              .resolve(path, &format!("{}/bad", request))
              .await;
          }
        },
      );
    },
  );

  if !skip_threaded {
    group.bench_with_input(
      BenchmarkId::from_parameter("[multi-threaded]resolve"),
      &data,
      |b, data| {
        let runner = multi_rt();
        let rspack_resolver = Arc::new(rspack_resolver(false));

        b.iter_with_setup(
          || {
            rspack_resolver.clear_cache();
          },
          |_| {
            runner.block_on(async {
              let mut join_set = JoinSet::new();
              data.iter().for_each(|(path, request)| {
                join_set.spawn(create_async_resolve_task(
                  rspack_resolver.clone(),
                  path.to_path_buf(),
                  request.to_string(),
                ));
              });
              let _ = join_set.join_all().await;
            });
          },
        );
      },
    );
  }

  group.bench_with_input(
    BenchmarkId::from_parameter("resolve from symlinks"),
    &symlinks_range,
    |b, data| {
      let runner = runtime::Runtime::new().expect("failed to create tokio runtime");
      let rspack_resolver = rspack_resolver(false);

      b.to_async(runner).iter_with_setup(
        || {
          rspack_resolver.clear_cache();
        },
        |_| async {
          for i in data.clone() {
            assert!(
              rspack_resolver
                .resolve(&symlink_test_dir, &format!("./file{i}"))
                .await
                .is_ok(),
              "file{i}.js"
            );
          }
        },
      );
    },
  );

  if !skip_threaded {
    group.bench_with_input(
      BenchmarkId::from_parameter("[multi-threaded]resolve from symlinks"),
      &symlinks_range,
      |b, data| {
        let runner = multi_rt();
        let rspack_resolver = Arc::new(rspack_resolver(false));

        let symlink_test_dir = symlink_test_dir.clone();

        b.to_async(runner).iter_with_setup(
          || {
            rspack_resolver.clear_cache();
          },
          |_| async {
            let mut join_set = JoinSet::new();

            data.clone().for_each(|i| {
              join_set.spawn(create_async_resolve_task(
                rspack_resolver.clone(),
                symlink_test_dir.clone(),
                format!("./file{i}").to_string(),
              ));
            });
            join_set.join_all().await;
          },
        );
      },
    );
  }

  let pnp_workspace = env::current_dir().unwrap().join("fixtures/pnp");
  let root_range = 1..11;

  group.bench_with_input(
    BenchmarkId::from_parameter("pnp resolve"),
    &root_range,
    |b, data| {
      let runner = runtime::Runtime::new().expect("failed to create tokio runtime");
      let rspack_resolver = Arc::new(rspack_resolver(true));

      b.iter_with_setup(
        || {
          // Drop all caches, then reload the PnP manifest before the timed
          // body runs. The manifest re-parse (~250KB regex compile) is
          // one-time work in real usage; keeping it out of the timed loop
          // lets resolver-level deltas surface.
          rspack_resolver.clear_cache();
          runner.block_on(async {
            let _ = rspack_resolver
              .resolve(pnp_workspace.join("1"), "preact")
              .await;
          });
        },
        |_| {
          runner.block_on(async {
            for i in data.clone() {
              let _ = rspack_resolver
                .resolve(pnp_workspace.join(format!("{i}")), "preact")
                .await;
            }
          });
        },
      );
    },
  );

  // tsconfig `paths` + project-references resolution. Each resolve hits the
  // warm `tsconfigs` cache once (a single key hash of the fixed config path),
  // so looping the case set keeps that lookup hot — this is the scenario that
  // exercises the `Utf8PathBuf` vs `PathBuf` map-key question.
  let tsconfig_dir = env::current_dir()
    .unwrap()
    .join("fixtures/tsconfig/cases/project_references");
  let tsconfig_data = vec![
    (tsconfig_dir.join("app"), "@/index.ts"),
    (tsconfig_dir.join("app"), "@/../index.ts"),
    (tsconfig_dir.join("project_a"), "@/index.ts"),
    (tsconfig_dir.join("project_b/src"), "@/index.ts"),
    (tsconfig_dir.join("project_a"), "./index.ts"),
    (tsconfig_dir.join("project_c"), "./index.ts"),
  ];

  // check validity
  runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
      let resolver = tsconfig_resolver();
      for (path, request) in &tsconfig_data {
        assert!(
          resolver.resolve(path, request).await.is_ok(),
          "tsconfig resolve failed {path:?} {request}, fixtures/tsconfig/cases/project_references"
        );
      }
    });

  group.bench_with_input(
    BenchmarkId::from_parameter("tsconfig resolve"),
    &tsconfig_data,
    |b, data| {
      let runner = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");
      let rspack_resolver = tsconfig_resolver();

      b.to_async(runner).iter_with_setup(
        || {
          rspack_resolver.clear_cache();
        },
        |_| async {
          for _ in 0..100 {
            for (path, request) in data {
              _ = rspack_resolver.resolve(path, request).await;
            }
          }
        },
      );
    },
  );
}

// Specifier microbenchmarks live in `benches/specifier.rs` (separate
// `[[bench]]` binary) so the very short `specifier/*` cases get a fresh
// instruction cache instead of competing with the resolver bench code for
// cache lines. See that file for the rationale.

criterion_group!(resolver, bench_resolver);
criterion_main!(resolver);

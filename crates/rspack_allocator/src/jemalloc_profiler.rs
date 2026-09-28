use std::{
  ffi::CString,
  path::PathBuf,
  sync::atomic::{AtomicUsize, Ordering},
};

static SNAPSHOT_INDEX: AtomicUsize = AtomicUsize::new(0);

/// Return the live bytes currently allocated by Rspack's jemalloc instance.
///
/// This is available only in the opt-in jemalloc profiling build. It does not
/// include allocations made by other allocators in the Node process.
pub fn allocated_bytes() -> Result<usize, String> {
  use tikv_jemalloc_ctl::{epoch, stats};

  epoch::advance().map_err(|error| format!("could not refresh jemalloc stats: {error:?}"))?;
  stats::allocated::read().map_err(|error| format!("could not read jemalloc stats: {error:?}"))
}

/// Write a jemalloc heap profile for the currently live Rust allocations.
///
/// The output is disabled unless `RSPACK_JEMALLOC_PROFILE_DIR` is set. Jemalloc profiling itself
/// must also be enabled at process startup with `_RJEM_MALLOC_CONF=prof:true`.
pub fn dump_compilation_heap_profile(kind: &str) {
  let Some(directory) = std::env::var_os("RSPACK_JEMALLOC_PROFILE_DIR") else {
    return;
  };

  if directory.is_empty() {
    return;
  }

  if let Err(error) = dump_profile(PathBuf::from(directory), kind) {
    eprintln!("[rspack-jemalloc] failed to write {kind} heap profile: {error}");
  }
}

fn dump_profile(directory: PathBuf, kind: &str) -> Result<(), String> {
  use tikv_jemalloc_ctl::raw;

  // SAFETY: `opt.prof` is a bool option in jemalloc's documented control namespace.
  let profiling_enabled = unsafe { raw::read::<bool>(b"opt.prof\0") }
    .map_err(|error| format!("could not read jemalloc profiling state: {error:?}"))?;
  if !profiling_enabled {
    return Err(
      "jemalloc profiling is disabled; set _RJEM_MALLOC_CONF=prof:true before starting Node".into(),
    );
  }

  std::fs::create_dir_all(&directory)
    .map_err(|error| format!("could not create {}: {error}", directory.display()))?;
  let index = SNAPSHOT_INDEX.fetch_add(1, Ordering::Relaxed);
  let profile_path = directory.join(format!("rspack-{kind}-{}-{index}.heap", std::process::id()));
  let path = CString::new(profile_path.to_string_lossy().as_bytes())
    .map_err(|error| format!("invalid profile path: {error}"))?;

  // SAFETY: `prof.dump` accepts a pointer to a null-terminated path. `path` stays alive for the
  // duration of the synchronous mallctl call.
  unsafe { raw::write(b"prof.dump\0", path.as_ptr()) }
    .map_err(|error| format!("could not dump profile: {error:?}"))?;

  eprintln!(
    "[rspack-jemalloc] wrote live heap profile to {}",
    profile_path.display()
  );
  Ok(())
}

/// Reserves output paths for a snapshot initiated by the binding.
pub struct AllocativeSnapshot {
  dir: std::path::PathBuf,
  name: String,
}

impl AllocativeSnapshot {
  pub fn new(name: &str) -> std::io::Result<Option<Self>> {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNT: AtomicUsize = AtomicUsize::new(0);
    let Some(dir) = std::env::var_os("RSPACK_ALLOCATIVE_DIR") else {
      return Ok(None);
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir)?;
    Ok(Some(Self {
      dir,
      name: format!("{}-{name}", COUNT.fetch_add(1, Ordering::Relaxed)),
    }))
  }

  pub fn heap_path(&self) -> std::path::PathBuf {
    self
      .dir
      .join("v8")
      .join(format!("{}.heapsnapshot", self.name))
  }

  /// Finish the one owner traversal, without visiting the compiler a second time.
  pub fn write(
    self,
    mut builder: crate::FlameGraphBuilder,
    mut counts: serde_json::Map<String, serde_json::Value>,
  ) -> std::io::Result<()> {
    let Self {
      dir,
      name: snapshot,
    } = self;
    builder.visit_global_roots();
    // Ustr handles borrow process-global interned strings. Charge the pool once
    // under its real owner, rather than once for each module/dependency handle.
    struct UstrStringArena;
    impl crate::Allocative for UstrStringArena {
      fn visit<'a, 'b: 'a>(&self, visitor: &'a mut crate::Visitor<'b>) {
        let used = ustr::total_allocated();
        let capacity = ustr::total_capacity();
        let mut root = visitor.enter(crate::Key::new("GlobalUstrStringArena"), 0);
        let mut arena = root.enter_unique(crate::Key::new("arena"), 0);
        arena.visit_simple(crate::Key::new("used"), used);
        arena.visit_simple(
          crate::Key::new("unused_capacity"),
          capacity.saturating_sub(used),
        );
        arena.exit();
        root.exit();
      }
    }
    builder.visit_root(&UstrStringArena);
    let output = builder.finish();
    std::fs::write(
      dir.join(format!("{snapshot}.objects.json")),
      serde_json::to_vec_pretty(
        &serde_json::json!({"snapshot": snapshot, "objects": output.objects()}),
      )?,
    )?;
    std::fs::write(
      dir.join(format!("{snapshot}.allocative")),
      output.flamegraph().write(),
    )?;
    std::fs::write(
      dir.join(format!("{snapshot}.warnings.txt")),
      output.warnings(),
    )?;

    let metadata = serde_json::json!({
      "snapshot": snapshot,
      "represented_bytes": output.flamegraph().total_size(),
      "shared_ownership": "first encountered owner; later references and cycles are deduplicated",
      "ustr_global_entries": ustr::num_entries(),
      "ustr_arena_used_bytes": ustr::total_allocated(),
      "ustr_arena_capacity_bytes": ustr::total_capacity(),
      "limits": ["Not RSS or peak heap usage", "Container allocator metadata and private hash-table overhead are not fully represented", "Ustr pool hash-index overhead and unreachable global interner entries outside the Ustr arena are not represented", "See warnings for inaccessible external state"]
    });
    std::fs::write(
      dir.join(format!("{snapshot}.metadata.json")),
      serde_json::to_vec_pretty(&metadata)?,
    )?;
    counts.insert("snapshot".into(), serde_json::json!(snapshot));
    std::fs::write(
      dir.join(format!("{snapshot}.counts.json")),
      serde_json::to_vec_pretty(&counts)?,
    )?;
    Ok(())
  }
}

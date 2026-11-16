use std::collections::BTreeMap;

mod count_labels;

/// Count graph entries separately from the byte traversal, without retaining targets.
pub(crate) fn module_graph_counts(
  compiler: &rspack_core::Compiler,
) -> serde_json::Map<String, serde_json::Value> {
  let graph = compiler.compilation.get_module_graph();
  let mut populations: BTreeMap<Vec<String>, usize> = BTreeMap::new();
  for (_, module) in graph.modules() {
    *populations
      .entry(vec![
        "Compiler".into(),
        "Compilation".into(),
        "modules".into(),
        match module.module_type() {
          rspack_core::ModuleType::Custom(name) => {
            count_labels::custom_module_type_label(name.as_str())
          }
          module_type => module_type.to_string(),
        },
      ])
      .or_default() += 1;
  }
  for (_, dependency) in graph.dependencies() {
    *populations
      .entry(vec![
        "Compiler".into(),
        "Compilation".into(),
        "dependencies".into(),
        format!("{:?}", dependency.dependency_type()),
      ])
      .or_default() += 1;
  }
  // An empty graph is a measured zero, not an unavailable population report.
  if populations.is_empty() {
    for population in ["modules", "dependencies"] {
      populations.insert(
        vec!["Compiler".into(), "Compilation".into(), population.into()],
        0,
      );
    }
  }
  let counts: Vec<_> = populations.into_iter().map(|(path, count)| serde_json::json!({
    "path": path, "count": count, "basis": "live entries in the current compilation module graph"
  })).collect();
  serde_json::Map::from_iter([
    ("unit".into(), serde_json::json!("entries")),
    (
      "scope".into(),
      serde_json::json!(
        "Current compilation after cache finalization and before compiler drop; module and dependency graph entries grouped by type"
      ),
    ),
    ("counts".into(), serde_json::json!(counts)),
  ])
}

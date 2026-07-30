use rspack_cacheable::cacheable;
use rspack_core::{Compilation, DependencyId};
use rspack_util::json_stringify_str;

/// Build-resolved identity of a mocked module, emitted as the trailing
/// argument of the generated `rstest_mock`/`rstest_unmock` call:
/// `{"o": <declaring file>, "r": <resolved target | null>}`.
///
/// `o` is the absolute path of the module declaring the `rs.mock` call,
/// captured at parse time. `r` is read from the module graph at
/// template-render time via the mocked target's `DependencyId`, so it
/// reflects the final resolution (aliases, extensions, externals): an
/// absolute file path for a bundled module, the external request (e.g. a
/// `node:` builtin spelling) for an external, or `null` when unresolved.
///
/// The runtime keys native (out-of-bundle) mocks by this identity instead of
/// re-deriving the resolution itself. Old runtimes ignore the extra argument.
#[cacheable]
#[derive(Debug, Clone)]
pub struct MockResolvedInfo {
  /// The mocked target's module dependency; its resolution is read at render time.
  pub target_dep: DependencyId,
  /// Absolute path of the module declaring the `rs.mock` call.
  pub origin_path: String,
}

impl MockResolvedInfo {
  pub fn render(&self, compilation: &Compilation) -> String {
    let module_graph = compilation.get_module_graph();
    let resolved = module_graph
      .get_module_by_dependency_id(&self.target_dep)
      .and_then(|module| {
        if let Some(normal) = module.as_normal_module() {
          // Absolute path only (no query/fragment): the runtime keys native
          // mocks by file, matching Node's resolution of the same specifier.
          normal
            .resource_resolved_data()
            .path()
            .map(|path| path.as_str())
        } else {
          module
            .as_external_module()
            .map(|external| external.get_request().primary())
        }
      });
    format!(
      "{{\"o\":{},\"r\":{}}}",
      json_stringify_str(&self.origin_path),
      resolved.map_or_else(|| "null".to_string(), json_stringify_str),
    )
  }

  /// Render the identity as the `", {json}"` trailing-argument segment, or
  /// `""` for `None`. The separator/position half of the contract lives here,
  /// next to the payload, so every emitting template shares one shape.
  pub fn render_trailing_arg(info: Option<&Self>, compilation: &Compilation) -> String {
    info.map_or_else(String::new, |info| {
      format!(", {}", info.render(compilation))
    })
  }
}

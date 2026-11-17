#![allow(clippy::unwrap_used)]

#[test]
fn context_filter_renderer_preserves_all_regex_spans() {
  rspack_tasks::within_compiler_context_for_testing_sync(|| {
    use std::path::{Path, PathBuf};

    use rspack_cacheable::{CacheableContext, ContextIdentifierLayout};
    use rspack_core::{
      ContextDependency, ContextModulePattern, ContextOptions, DependencyRange, ModuleIdentifier,
    };
    use rspack_plugin_javascript::dependency::ImportContextDependency;
    use rspack_regex::RspackRegex;
    struct Context {
      root: PathBuf,
      identifier: String,
      layout: ContextIdentifierLayout,
    }
    impl CacheableContext for Context {
      fn project_root(&self) -> Option<&Path> {
        Some(&self.root)
      }
      fn context_identifier_layout(&self, identifier: &str) -> Option<&ContextIdentifierLayout> {
        (identifier == self.identifier).then_some(&self.layout)
      }
    }
    let root = if cfg!(windows) {
      "C:/project"
    } else {
      "/project"
    };
    let moved = if cfg!(windows) { "D:/moved" } else { "/moved" };
    let pattern = RspackRegex::new(r".*[\\/]a\.js$").unwrap();
    let include = RspackRegex::new(&format!("prefix|{root}/allowed")).unwrap();
    let exclude = RspackRegex::new(&format!("{root}/denied|other")).unwrap();
    let options = ContextOptions {
      compiler_context: root.into(),
      request: "./dir".into(),
      pattern: ContextModulePattern::RegExp(pattern.clone()),
      include: Some(include.clone()),
      exclude: Some(exclude.clone()),
      ..Default::default()
    };
    let dependency = ImportContextDependency::new(
      options,
      DependencyRange::new(0, 1),
      DependencyRange::new(0, 1),
      false,
    );
    let identifier = ContextDependency::resource_identifier(&dependency).to_owned();
    let regexes = dependency.resource_identifier_regex_ranges();
    assert_eq!(regexes.len(), 3);
    for (range, regex) in regexes.iter().zip([&pattern, &include, &exclude]) {
      assert_eq!(&identifier[range.clone()], regex.to_source_string());
    }
    let context = Context {
      root: root.into(),
      identifier: identifier.clone(),
      layout: ContextIdentifierLayout {
        regexes,
        paths: Vec::new(),
      },
    };
    let value = ModuleIdentifier::from(identifier.as_str());
    let bytes = rspack_cacheable::to_bytes(&value, &context).unwrap();
    let reader = Context {
      root: moved.into(),
      identifier: String::new(),
      layout: Default::default(),
    };
    let restored: ModuleIdentifier = rspack_cacheable::from_bytes(&bytes, &reader).unwrap();
    assert_eq!(restored.as_str(), identifier);
  });
}

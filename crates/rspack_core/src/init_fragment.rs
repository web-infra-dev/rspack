use std::{
  collections::{BTreeMap, BTreeSet},
  fmt::Debug,
  hash::{BuildHasherDefault, Hash},
  sync::atomic::AtomicU32,
};

use dyn_clone::{clone_trait_object, DynClone};
use hashlink::LinkedHashSet;
use indexmap::IndexMap;
use rspack_error::Result;
use rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_util::ext::{DynHash, IntoAny};
use rustc_hash::FxHasher;
use swc_core::ecma::atoms::Atom;

use crate::{
  merge_runtime, property_name, runtime_condition_expression, ExportsArgument, GenerateContext,
  RuntimeCondition, RuntimeGlobals,
};

static NEXT_INIT_FRAGMENT_KEY_UNIQUE_ID: AtomicU32 = AtomicU32::new(0);

pub struct InitFragmentContents {
  pub start: String,
  pub end: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum InitFragmentKey {
  Unique(u32),
  ESMImport(String),
  ESMExportStar(String), // TODO: align with webpack and remove this
  ESMExports,
  CommonJsExports(String),
  ModuleExternal(String),
  ExternalModule(String),
  AwaitDependencies,
  ESMCompatibility,
  ModuleDecorator(String /* module_id */),
  ESMFakeNamespaceObjectFragment(String),
  Const(String),
}

impl InitFragmentKey {
  pub fn unique() -> Self {
    Self::Unique(
      NEXT_INIT_FRAGMENT_KEY_UNIQUE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    )
  }
}

impl InitFragmentKey {
  pub fn merge_fragments<C: InitFragmentRenderContext>(
    &self,
    fragments: Vec<Box<dyn InitFragment<C>>>,
  ) -> Box<dyn InitFragment<C>> {
    match self {
      InitFragmentKey::ESMImport(_) => {
        let mut iter = fragments.into_iter();
        let first = iter
          .next()
          .expect("keyed_fragments should at least have one value");
        let first = first
          .into_any()
          .downcast::<ConditionalInitFragment>()
          .expect("fragment of InitFragmentKey::ESMImport should be a ConditionalInitFragment");

        if matches!(first.runtime_condition, RuntimeCondition::Boolean(true)) {
          return first;
        }

        let mut res = first;
        for fragment in iter {
          let fragment = fragment
            .into_any()
            .downcast::<ConditionalInitFragment>()
            .expect("fragment of InitFragmentKey::ESMImport should be a ConditionalInitFragment");
          res = ConditionalInitFragment::merge(res, fragment);
          if matches!(res.runtime_condition, RuntimeCondition::Boolean(true)) {
            return res;
          }
        }
        res
      }
      InitFragmentKey::ESMExports => {
        let mut export_map: Vec<(Atom, Atom)> = vec![];
        let mut iter = fragments.into_iter();
        let first = iter
          .next()
          .expect("keyed_fragments should at least have one value");
        let first = first
          .into_any()
          .downcast::<ESMExportInitFragment>()
          .expect("fragment of InitFragmentKey::ESMExports should be a ESMExportInitFragment");
        let export_argument = first.exports_argument;
        export_map.extend(first.export_map);
        for fragment in iter {
          let fragment = fragment
            .into_any()
            .downcast::<ESMExportInitFragment>()
            .expect("fragment of InitFragmentKey::ESMExports should be a ESMExportInitFragment");
          export_map.extend(fragment.export_map);
        }
        ESMExportInitFragment::new(export_argument, export_map).boxed()
      }
      InitFragmentKey::AwaitDependencies => {
        let promises = fragments.into_iter().map(|f| f.into_any().downcast::<AwaitDependenciesInitFragment>().expect("fragment of InitFragmentKey::AwaitDependencies should be a AwaitDependenciesInitFragment")).flat_map(|f| f.promises).collect();
        AwaitDependenciesInitFragment::new(promises).boxed()
      }
      InitFragmentKey::ExternalModule(_) => {
        let mut iter = fragments.into_iter();
        let first = iter
          .next()
          .expect("keyed_fragments should at least have one value");

        let first = first
          .into_any()
          .downcast::<ExternalModuleInitFragment>()
          .expect(
            "fragment of InitFragmentKey::ExternalModule should be a ExternalModuleInitFragment",
          );

        let mut res = first;
        for fragment in iter {
          let fragment = fragment
            .into_any()
            .downcast::<ExternalModuleInitFragment>()
            .expect(
              "fragment of InitFragmentKey::ExternalModule should be a ExternalModuleInitFragment",
            );
          res = ExternalModuleInitFragment::merge(*res, *fragment);
        }
        res
      }
      InitFragmentKey::ESMFakeNamespaceObjectFragment(_)
      | InitFragmentKey::ESMExportStar(_)
      | InitFragmentKey::ModuleExternal(_)
      | InitFragmentKey::ModuleDecorator(_)
      | InitFragmentKey::CommonJsExports(_)
      | InitFragmentKey::Const(_) => first(fragments),
      InitFragmentKey::ESMCompatibility | InitFragmentKey::Unique(_) => {
        debug_assert!(fragments.len() == 1, "fragment = {self:?}");
        first(fragments)
      }
    }
  }
}

fn first<C>(fragments: Vec<Box<dyn InitFragment<C>>>) -> Box<dyn InitFragment<C>> {
  fragments
    .into_iter()
    .next()
    .expect("should at least have one fragment")
}

pub trait InitFragmentRenderContext {
  fn add_runtime_requirements(&mut self, requirement: RuntimeGlobals);
  fn runtime_condition_expression(&mut self, runtime_condition: &RuntimeCondition) -> String;
}

pub trait InitFragment<C>: IntoAny + DynHash + DynClone + Debug + Sync + Send {
  /// getContent + getEndContent
  fn contents(self: Box<Self>, context: &mut C) -> Result<InitFragmentContents>;

  fn stage(&self) -> InitFragmentStage;

  fn position(&self) -> i32;

  fn key(&self) -> &InitFragmentKey;
}

clone_trait_object!(InitFragment<GenerateContext<'_>>);
clone_trait_object!(InitFragment<ChunkRenderContext>);

impl<C> Hash for dyn InitFragment<C> + '_ {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.dyn_hash(state)
  }
}

pub trait InitFragmentExt<C> {
  fn boxed(self) -> Box<dyn InitFragment<C>>;
}

impl<C, T: InitFragment<C> + 'static> InitFragmentExt<C> for T {
  fn boxed(self) -> Box<dyn InitFragment<C>> {
    Box::new(self)
  }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum InitFragmentStage {
  StageConstants,
  StageAsyncBoundary,
  StageESMExports,
  StageESMImports,
  StageProvides,
  StageAsyncDependencies,
  StageAsyncESMImports,
}

/// InitFragment.addToSource
pub fn render_init_fragments<C: InitFragmentRenderContext>(
  source: BoxSource,
  mut fragments: Vec<Box<dyn InitFragment<C>>>,
  context: &mut C,
) -> Result<BoxSource> {
  // here use sort_by_key because need keep order equal stage fragments
  fragments.sort_by(|a, b| {
    let stage = a.stage().cmp(&b.stage());
    if !stage.is_eq() {
      return stage;
    }
    a.position().cmp(&b.position())
  });

  let mut keyed_fragments: IndexMap<
    InitFragmentKey,
    Vec<Box<dyn InitFragment<C>>>,
    BuildHasherDefault<FxHasher>,
  > = IndexMap::default();
  for fragment in fragments {
    let key = fragment.key();
    if let Some(value) = keyed_fragments.get_mut(key) {
      value.push(fragment);
    } else {
      keyed_fragments.insert(key.clone(), vec![fragment]);
    }
  }

  let mut end_contents = vec![];
  let mut concat_source = ConcatSource::default();

  for (key, fragments) in keyed_fragments {
    let f = key.merge_fragments(fragments);
    let contents = f.contents(context)?;
    concat_source.add(RawSource::from(contents.start));
    if let Some(end_content) = contents.end {
      end_contents.push(RawSource::from(end_content))
    }
  }

  concat_source.add(source);

  for content in end_contents.into_iter().rev() {
    concat_source.add(content);
  }

  Ok(concat_source.boxed())
}

pub type BoxInitFragment<C> = Box<dyn InitFragment<C>>;
pub type BoxModuleInitFragment<'a> = BoxInitFragment<GenerateContext<'a>>;
pub type BoxChunkInitFragment = BoxInitFragment<ChunkRenderContext>;
pub type ModuleInitFragments<'a> = Vec<BoxModuleInitFragment<'a>>;
pub type ChunkInitFragments = Vec<BoxChunkInitFragment>;

impl InitFragmentRenderContext for GenerateContext<'_> {
  fn add_runtime_requirements(&mut self, requirement: RuntimeGlobals) {
    self.runtime_requirements.insert(requirement);
  }

  fn runtime_condition_expression(&mut self, runtime_condition: &RuntimeCondition) -> String {
    runtime_condition_expression(
      &self.compilation.chunk_graph,
      Some(runtime_condition),
      self.runtime,
      self.runtime_requirements,
    )
  }
}

pub struct ChunkRenderContext;

impl InitFragmentRenderContext for ChunkRenderContext {
  fn add_runtime_requirements(&mut self, _requirement: RuntimeGlobals) {
    unreachable!("should not add runtime requirements in chunk render context")
  }

  fn runtime_condition_expression(&mut self, _runtime_condition: &RuntimeCondition) -> String {
    unreachable!("should not call runtime condition expression in chunk render context")
  }
}

#[derive(Debug, Clone, Hash)]
pub struct NormalInitFragment {
  content: String,
  stage: InitFragmentStage,
  position: i32,
  key: InitFragmentKey,
  end_content: Option<String>,
}

impl NormalInitFragment {
  pub fn new(
    content: String,
    stage: InitFragmentStage,
    position: i32,
    key: InitFragmentKey,
    end_content: Option<String>,
  ) -> Self {
    NormalInitFragment {
      content,
      stage,
      position,
      key,
      end_content,
    }
  }
}

impl<C> InitFragment<C> for NormalInitFragment {
  fn contents(self: Box<Self>, _context: &mut C) -> Result<InitFragmentContents> {
    Ok(InitFragmentContents {
      start: self.content,
      end: self.end_content,
    })
  }

  fn stage(&self) -> InitFragmentStage {
    self.stage
  }

  fn position(&self) -> i32 {
    self.position
  }

  fn key(&self) -> &InitFragmentKey {
    &self.key
  }
}

#[derive(Debug, Clone, Hash)]
pub struct ESMExportInitFragment {
  exports_argument: ExportsArgument,
  // TODO: should be a map
  export_map: Vec<(Atom, Atom)>,
}

impl ESMExportInitFragment {
  pub fn new(exports_argument: ExportsArgument, export_map: Vec<(Atom, Atom)>) -> Self {
    Self {
      exports_argument,
      export_map,
    }
  }
}

impl<C: InitFragmentRenderContext> InitFragment<C> for ESMExportInitFragment {
  fn contents(mut self: Box<Self>, context: &mut C) -> Result<InitFragmentContents> {
    context.add_runtime_requirements(RuntimeGlobals::EXPORTS);
    context.add_runtime_requirements(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    self.export_map.sort_by(|a, b| a.0.cmp(&b.0));
    let exports = format!(
      "{{\n  {}\n}}",
      self
        .export_map
        .iter()
        .map(|s| {
          let prop = property_name(&s.0)?;
          Ok(format!("{}: function() {{ return {}; }}", prop, s.1))
        })
        .collect::<Result<Vec<_>>>()?
        .join(",\n  ")
    );

    Ok(InitFragmentContents {
      start: format!(
        "{}({}, {});\n",
        RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
        self.exports_argument,
        exports
      ),
      end: None,
    })
  }

  fn stage(&self) -> InitFragmentStage {
    InitFragmentStage::StageESMExports
  }

  fn position(&self) -> i32 {
    1
  }

  fn key(&self) -> &InitFragmentKey {
    &InitFragmentKey::ESMExports
  }
}

#[derive(Debug, Clone, Hash)]
pub struct AwaitDependenciesInitFragment {
  promises: LinkedHashSet<String, BuildHasherDefault<FxHasher>>,
}

impl AwaitDependenciesInitFragment {
  pub fn new(promises: LinkedHashSet<String, BuildHasherDefault<FxHasher>>) -> Self {
    Self { promises }
  }

  pub fn new_single(promise: String) -> Self {
    let mut promises = LinkedHashSet::default();
    promises.insert(promise);
    Self { promises }
  }
}

impl<C: InitFragmentRenderContext> InitFragment<C> for AwaitDependenciesInitFragment {
  fn contents(self: Box<Self>, context: &mut C) -> Result<InitFragmentContents> {
    context.add_runtime_requirements(RuntimeGlobals::MODULE);
    if self.promises.is_empty() {
      Ok(InitFragmentContents {
        start: String::new(),
        end: None,
      })
    } else if self.promises.len() == 1 {
      let sep = self.promises.front().expect("at least have one");
      Ok(InitFragmentContents {
        start: format!(
          "var __webpack_async_dependencies__ = __webpack_handle_async_dependencies__([{sep}]);\n{sep} = (__webpack_async_dependencies__.then ? (await __webpack_async_dependencies__)() : __webpack_async_dependencies__)[0];"
        ),
        end: None,
      })
    } else {
      let sep = Vec::from_iter(self.promises).join(", ");
      Ok(InitFragmentContents {
        start: format!(
          "var __webpack_async_dependencies__ = __webpack_handle_async_dependencies__([{sep}]);\n([{sep}] = __webpack_async_dependencies__.then ? (await __webpack_async_dependencies__)() : __webpack_async_dependencies__);"
        ),
        end: None,
      })
    }
  }

  fn stage(&self) -> InitFragmentStage {
    InitFragmentStage::StageAsyncDependencies
  }

  fn position(&self) -> i32 {
    0
  }

  fn key(&self) -> &InitFragmentKey {
    &InitFragmentKey::AwaitDependencies
  }
}

#[derive(Debug, Clone, Hash)]
pub struct ConditionalInitFragment {
  content: String,
  stage: InitFragmentStage,
  position: i32,
  key: InitFragmentKey,
  end_content: Option<String>,
  runtime_condition: RuntimeCondition,
}

impl ConditionalInitFragment {
  pub fn new(
    content: String,
    stage: InitFragmentStage,
    position: i32,
    key: InitFragmentKey,
    end_content: Option<String>,
    runtime_condition: RuntimeCondition,
  ) -> Self {
    ConditionalInitFragment {
      content,
      stage,
      position,
      key,
      end_content,
      runtime_condition,
    }
  }

  pub fn merge(
    one: Box<ConditionalInitFragment>,
    other: Box<ConditionalInitFragment>,
  ) -> Box<ConditionalInitFragment> {
    if matches!(one.runtime_condition, RuntimeCondition::Boolean(true)) {
      return one;
    }
    if matches!(other.runtime_condition, RuntimeCondition::Boolean(true)) {
      return other;
    }
    if matches!(one.runtime_condition, RuntimeCondition::Boolean(false)) {
      return other;
    }
    if matches!(other.runtime_condition, RuntimeCondition::Boolean(false)) {
      return one;
    }
    Box::new(Self {
      content: one.content,
      stage: one.stage,
      position: one.position,
      key: one.key,
      end_content: one.end_content,
      runtime_condition: RuntimeCondition::Spec(merge_runtime(
        one.runtime_condition.as_spec().expect("should be spec"),
        other.runtime_condition.as_spec().expect("should be spec"),
      )),
    })
  }
}

impl<C: InitFragmentRenderContext> InitFragment<C> for ConditionalInitFragment {
  fn contents(self: Box<Self>, context: &mut C) -> Result<InitFragmentContents> {
    Ok(
      if matches!(self.runtime_condition, RuntimeCondition::Boolean(false))
        || self.content.is_empty()
      {
        InitFragmentContents {
          start: String::new(),
          end: Some(String::new()),
        }
      } else if matches!(self.runtime_condition, RuntimeCondition::Boolean(true)) {
        InitFragmentContents {
          start: self.content,
          end: self.end_content,
        }
      } else {
        let condition = context.runtime_condition_expression(&self.runtime_condition);
        if condition == "true" {
          InitFragmentContents {
            start: self.content,
            end: self.end_content,
          }
        } else {
          InitFragmentContents {
            start: wrap_in_condition(&condition, &self.content),
            end: self.end_content.map(|c| wrap_in_condition(&condition, &c)),
          }
        }
      },
    )
  }

  fn stage(&self) -> InitFragmentStage {
    self.stage
  }

  fn position(&self) -> i32 {
    self.position
  }

  fn key(&self) -> &InitFragmentKey {
    &self.key
  }
}

fn wrap_in_condition(condition: &str, source: &str) -> String {
  format!(
    r#"if ({condition}) {{
  {source}
}}"#
  )
}

#[derive(Debug, Clone, Hash)]
pub struct ExternalModuleInitFragment {
  imported_module: String,
  // webpack also supports `ImportSpecifiers` but not ever used.
  import_specifiers: BTreeMap<String, BTreeSet<String>>,
  default_import: Option<String>,
  stage: InitFragmentStage,
  position: i32,
  key: InitFragmentKey,
}

impl ExternalModuleInitFragment {
  pub fn new(
    imported_module: String,
    import_specifiers: Vec<(String, String)>,
    default_import: Option<String>,
    stage: InitFragmentStage,
    position: i32,
  ) -> Self {
    let mut self_import_specifiers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for (name, value) in import_specifiers {
      if let Some(set) = self_import_specifiers.get_mut(&name) {
        set.insert(value);
      } else {
        let mut set = BTreeSet::new();
        set.insert(value);
        self_import_specifiers.insert(name, set);
      }
    }

    let key = InitFragmentKey::ExternalModule(format!(
      "external module imports|{}|{}",
      imported_module,
      default_import.clone().unwrap_or_else(|| "null".to_string()),
    ));

    Self {
      imported_module,
      import_specifiers: self_import_specifiers,
      default_import,
      stage,
      position,
      key,
    }
  }

  pub fn merge(
    one: ExternalModuleInitFragment,
    other: ExternalModuleInitFragment,
  ) -> Box<ExternalModuleInitFragment> {
    let mut import_specifiers = one.import_specifiers.clone();
    for (name, value) in other.import_specifiers {
      if let Some(set) = import_specifiers.get_mut(&name) {
        set.extend(value);
      } else {
        import_specifiers.insert(name, value);
      }
    }

    Box::new(Self {
      imported_module: one.imported_module,
      import_specifiers,
      default_import: one.default_import,
      stage: one.stage,
      position: one.position,
      key: one.key,
    })
  }
}

impl<C: InitFragmentRenderContext> InitFragment<C> for ExternalModuleInitFragment {
  fn contents(self: Box<Self>, _context: &mut C) -> Result<InitFragmentContents> {
    let mut named_imports = vec![];

    for (name, specifiers) in self.import_specifiers {
      for spec in specifiers {
        if name == spec {
          named_imports.push(spec);
        } else {
          named_imports.push(format!("{name} as {spec}"));
        }
      }
    }

    let mut imports_string: String;
    imports_string = if named_imports.is_empty() {
      String::new()
    } else {
      format!("{{{}}}", named_imports.join(", "))
    };

    if let Some(default_import) = self.default_import {
      imports_string = format!(
        "{}{}",
        default_import,
        if imports_string.is_empty() {
          String::new()
        } else {
          format!(", {imports_string}")
        }
      );
    }

    let start = format!(
      "import {} from {};\n",
      imports_string,
      serde_json::to_string(&self.imported_module).expect("invalid json tostring")
    );

    Ok(InitFragmentContents { start, end: None })
  }

  fn stage(&self) -> InitFragmentStage {
    self.stage
  }

  fn position(&self) -> i32 {
    self.position
  }

  fn key(&self) -> &InitFragmentKey {
    &self.key
  }
}

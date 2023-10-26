use std::{
  fmt::Debug,
  hash::{BuildHasherDefault, Hash},
};

use dyn_clone::{clone_trait_object, DynClone};
use hashlink::LinkedHashSet;
use indexmap::IndexMap;
use rspack_error::Result;
use rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_util::ext::{DynHash, IntoAny};
use rustc_hash::FxHasher;
use swc_core::ecma::atoms::JsWord;

use crate::{property_name, ExportsArgument, GenerateContext, RuntimeGlobals};

pub struct InitFragmentContents {
  pub start: String,
  pub end: Option<String>,
}

pub struct InitFragmentKeyUniqie;
pub type InitFragmentKeyUKey = rspack_database::Ukey<InitFragmentKeyUniqie>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum InitFragmentKey {
  HarmonyImport(String),
  HarmonyExportStar(String), // TODO: align with webpack and remove this
  HarmonyExports,
  ExternalModule(String),
  AwaitDependencies,
  HarmonyCompatibility,
  ModuleDecorator(String /* module_id */),
  Uniqie(InitFragmentKeyUKey),
}

impl InitFragmentKey {
  pub fn uniqie() -> Self {
    Self::Uniqie(rspack_database::Ukey::new())
  }
}

impl InitFragmentKey {
  pub fn merge_fragments<C: InitFragmentRenderContext>(
    &self,
    fragments: Vec<Box<dyn InitFragment<C>>>,
  ) -> Box<dyn InitFragment<C>> {
    match self {
      InitFragmentKey::HarmonyExports => {
        let mut export_map = vec![];
        let mut iter = fragments.into_iter();
        let first = iter
          .next()
          .expect("keyed_fragments should at least have one value");
        let first = first
          .into_any()
          .downcast::<HarmonyExportInitFragment>()
          .expect(
            "fragment of InitFragmentKey::HarmonyExports should be a HarmonyExportInitFragment",
          );
        let export_argument = first.exports_argument;
        export_map.extend(first.export_map);
        for fragment in iter {
          let fragment = fragment
            .into_any()
            .downcast::<HarmonyExportInitFragment>()
            .expect(
              "fragment of InitFragmentKey::HarmonyExports should be a HarmonyExportInitFragment",
            );
          export_map.extend(fragment.export_map);
        }
        HarmonyExportInitFragment::new(export_argument, export_map).boxed()
      }
      InitFragmentKey::AwaitDependencies => {
        let promises = fragments.into_iter().map(|f| f.into_any().downcast::<AwaitDependenciesInitFragment>().expect("fragment of InitFragmentKey::AwaitDependencies should be a AwaitDependenciesInitFragment")).flat_map(|f| f.promises).collect();
        AwaitDependenciesInitFragment::new(promises).boxed()
      }
      InitFragmentKey::HarmonyImport(_)
      | InitFragmentKey::HarmonyExportStar(_)
      | InitFragmentKey::ExternalModule(_)
      | InitFragmentKey::ModuleDecorator(_) => first(fragments),
      InitFragmentKey::HarmonyCompatibility | InitFragmentKey::Uniqie(_) => {
        debug_assert!(fragments.len() == 1, "fragment = {:?}", self);
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
  StageHarmonyExports,
  StageHarmonyImports,
  StageProvides,
  StageAsyncDependencies,
  StageAsyncHarmonyImports,
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
}

pub struct ChunkRenderContext;

impl InitFragmentRenderContext for ChunkRenderContext {
  fn add_runtime_requirements(&mut self, _requirement: RuntimeGlobals) {
    unreachable!("should not add runtime requirements in chunk render context")
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
pub struct HarmonyExportInitFragment {
  exports_argument: ExportsArgument,
  // TODO: should be a map
  export_map: Vec<(JsWord, JsWord)>,
}

impl HarmonyExportInitFragment {
  pub fn new(exports_argument: ExportsArgument, export_map: Vec<(JsWord, JsWord)>) -> Self {
    Self {
      exports_argument,
      export_map,
    }
  }
}

impl<C: InitFragmentRenderContext> InitFragment<C> for HarmonyExportInitFragment {
  fn contents(self: Box<Self>, context: &mut C) -> Result<InitFragmentContents> {
    context.add_runtime_requirements(RuntimeGlobals::EXPORTS);
    context.add_runtime_requirements(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);

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
    InitFragmentStage::StageHarmonyExports
  }

  fn position(&self) -> i32 {
    1
  }

  fn key(&self) -> &InitFragmentKey {
    &InitFragmentKey::HarmonyExports
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
        start: "".to_string(),
        end: None,
      })
    } else {
      let sep = Vec::from_iter(self.promises.into_iter()).join(", ");
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

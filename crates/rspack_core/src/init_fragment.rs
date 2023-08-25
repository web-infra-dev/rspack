use rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_util::ext::AsAny;
use rustc_hash::FxHashMap as HashMap;
use swc_core::ecma::atoms::JsWord;

use crate::{ExportsArgument, RuntimeGlobals};

pub trait InitFragment: AsAny {}

pub type BoxInitFragment = Box<dyn InitFragment>;

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct NormalInitFragment {
  pub content: String,
  pub stage: InitFragmentStage,
  pub end_content: Option<Box<String>>,
}

impl NormalInitFragment {
  pub fn new(content: String, stage: InitFragmentStage, end_content: Option<Box<String>>) -> Self {
    NormalInitFragment {
      content,
      stage,
      end_content,
    }
  }
}

impl InitFragment for NormalInitFragment {}

#[derive(Debug)]
pub struct HarmonyExportInitFragment {
  pub export_map: HashMap<JsWord, JsWord>,
}

impl InitFragment for HarmonyExportInitFragment {}

macro_rules! impl_init_fragment_downcast_helpers {
  ($ty:ty, $ident:ident) => {
    impl dyn InitFragment + '_ {
      ::paste::paste! {
        pub fn [<as_ $ident>](&self) -> Option<& $ty> {
          self.as_any().downcast_ref::<$ty>()
        }

        pub fn [<as_ $ident _mut>](&mut self) -> Option<&mut $ty> {
          self.as_any_mut().downcast_mut::<$ty>()
        }
      }
    }
  };
}

impl_init_fragment_downcast_helpers!(NormalInitFragment, normal_init_fragment);
impl_init_fragment_downcast_helpers!(HarmonyExportInitFragment, harmony_export_init_fragment);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum InitFragmentStage {
  StageConstants,
  StageAsyncBoundary,
  StageHarmonyExportsCompatibility,
  StageHarmonyExports,
  StageHarmonyImports,
  StageProvides,
  StageAsyncDependencies,
  StageAsyncHarmonyImports,
}

pub type ChunkInitFragments = HashMap<String, NormalInitFragment>;

pub fn render_init_fragments(
  source: BoxSource,
  fragments: &mut Vec<&mut NormalInitFragment>,
) -> BoxSource {
  // here use sort_by_key because need keep order equal stage fragments
  fragments.sort_by_key(|m| m.stage);
  // merge same init fragments
  fragments.dedup();

  let mut sources = vec![];

  fragments.iter_mut().for_each(|f| {
    sources.push(RawSource::from(std::mem::take(&mut f.content)).boxed());
  });

  sources.push(source);

  fragments.iter_mut().rev().for_each(|f| {
    if let Some(box end_content) = std::mem::take(&mut f.end_content) {
      sources.push(RawSource::from(end_content).boxed());
    }
  });

  ConcatSource::new(sources).boxed()
}

pub fn render_box_init_fragments(
  mut fragments: Vec<BoxInitFragment>,
  source: BoxSource,
  exports_argument: ExportsArgument,
  runtime_requirements: &mut RuntimeGlobals,
) -> BoxSource {
  let mut normal_init_fragments = vec![];

  let mut harmony_export_init_fragments =
    merge_harmony_export_init_fragments(&mut fragments, exports_argument, runtime_requirements);
  if let Some(fragment) = harmony_export_init_fragments.as_mut() {
    normal_init_fragments.push(fragment);
  }

  normal_init_fragments.extend(
    fragments
      .iter_mut()
      .filter_map(|f| f.as_normal_init_fragment_mut())
      .collect::<Vec<_>>(),
  );
  render_init_fragments(source, &mut normal_init_fragments)
}

fn merge_harmony_export_init_fragments(
  fragments: &mut [BoxInitFragment],
  exports_argument: ExportsArgument,
  runtime_requirements: &mut RuntimeGlobals,
) -> Option<NormalInitFragment> {
  let mut export_map = HashMap::default();
  fragments.iter_mut().for_each(|f| {
    if let Some(f) = f.as_harmony_export_init_fragment_mut() {
      export_map.extend(std::mem::take(&mut f.export_map))
    }
  });

  if !export_map.is_empty() {
    runtime_requirements.insert(RuntimeGlobals::EXPORTS);
    runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);

    return Some(NormalInitFragment::new(
      format!("{}({exports_argument}, {});\n", "", ""),
      InitFragmentStage::StageHarmonyExports,
      None,
    ));
  }

  None
}

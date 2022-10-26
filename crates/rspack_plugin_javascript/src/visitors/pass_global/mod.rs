use crate::utils::get_swc_compiler;
use anyhow::Error;
use swc_common::{errors::Handler, Globals, Mark, GLOBALS};
use swc_ecma_transforms::helpers::Helpers;

pub struct PassGlobal {
  pub global: Globals,
  pub helper: Helpers,
  pub top_level_mark: Mark,
  pub unresolved_mark: Mark,
}

impl PassGlobal {
  pub fn new() -> Self {
    let global: Globals = Default::default();
    // top_level_mark is mark(1)
    // unresolved_mark is mark(2)
    // helper_mark is mark(3)
    let (top_level_mark, unresolved_mark, helper) = GLOBALS.set(&global, || {
      get_swc_compiler().run(|| (Mark::new(), Mark::new(), Helpers::new(true)))
    });
    Self {
      global,
      helper,
      top_level_mark,
      unresolved_mark,
    }
  }

  pub fn try_with_handler<F, Ret>(&self, f: F) -> Result<Ret, Error>
  where
    F: FnOnce(&Handler) -> Result<Ret, Error>,
  {
    GLOBALS.set(&self.global, || {
      swc_ecma_transforms::helpers::HELPERS.set(&self.helper, || {
        swc::try_with_handler(get_swc_compiler().cm.clone(), Default::default(), f)
      })
    })
  }
}

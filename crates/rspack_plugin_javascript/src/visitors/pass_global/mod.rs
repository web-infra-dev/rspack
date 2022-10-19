use crate::utils::get_swc_compiler;
use anyhow::Error;
use once_cell::sync::Lazy;
use std::sync::Arc;
use swc_common::{errors::Handler, Globals, Mark, GLOBALS};
use swc_ecma_transforms::helpers::Helpers;

static PASS_GLOBAL: Lazy<Arc<PassGlobal>> = Lazy::new(|| {
  let global: Globals = Default::default();
  let (top_level_mark, unresolved_mark) = GLOBALS.set(&global, || {
    get_swc_compiler().run(|| (Mark::new(), Mark::new()))
  });
  Arc::new(PassGlobal {
    top_level_mark,
    unresolved_mark,
  })
});

pub struct PassGlobal {
  pub top_level_mark: Mark,
  pub unresolved_mark: Mark,
}

impl PassGlobal {
  pub fn new() -> Arc<Self> {
    PASS_GLOBAL.clone()
  }

  pub fn try_with_handler<F, Ret>(&self, external: bool, f: F) -> Result<Ret, Error>
  where
    F: FnOnce(&Handler) -> Result<Ret, Error>,
  {
    GLOBALS.set(&Default::default(), || {
      swc_ecma_transforms::helpers::HELPERS.set(&Helpers::new(external), || {
        swc::try_with_handler(get_swc_compiler().cm.clone(), Default::default(), f)
      })
    })
  }
}

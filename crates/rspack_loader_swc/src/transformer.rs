use either::Either;
use rustc_hash::FxHashMap;
use swc_core::atoms::Atom;
use swc_core::common::BytePos;
use swc_core::ecma::ast::Pass;
use swc_core::ecma::ast::{noop_pass, Ident};
use swc_core::ecma::visit::{noop_visit_type, Visit};

use crate::options::RspackExperiments;

macro_rules! either {
  ($config:expr, $f:expr) => {
    if let Some(config) = &$config {
      #[allow(clippy::redundant_closure_call)]
      Either::Left($f(config))
    } else {
      Either::Right(noop_pass())
    }
  };
  ($config:expr, $f:expr, $enabled:expr) => {
    if $enabled {
      either!($config, $f)
    } else {
      Either::Right(noop())
    }
  };
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn transform(rspack_experiments: &RspackExperiments) -> impl Pass + '_ {
  either!(rspack_experiments.import, |options| {
    rspack_swc_plugin_import::plugin_import(options)
  })
}

pub struct IdentCollector {
  pub names: FxHashMap<BytePos, Atom>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}

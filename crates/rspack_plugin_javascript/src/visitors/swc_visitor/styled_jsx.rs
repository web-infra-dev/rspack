use std::sync::Arc;

use swc_core::common::{FileName, SourceMap};
use swc_core::ecma::visit::Fold;

pub fn styled_jsx(cm: &Arc<SourceMap>) -> impl Fold {
  styled_jsx::visitor::styled_jsx(cm.clone(), FileName::Anon)
}

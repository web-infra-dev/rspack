use std::collections::HashMap;
use std::sync::Arc;

use rspack_error::miette::{Diagnostic, Severity};
use rspack_error::TraceableError;
use swc_core::common::collections::AHashMap;
use swc_core::common::{FileName, Spanned};
use swc_core::ecma::ast::EsVersion;
use swc_core::ecma::parser::{parse_file_as_expr, EsConfig, Syntax};
use swc_core::ecma::transforms::optimization::inline_globals2;
use swc_core::ecma::utils::NodeIgnoringSpan;
use swc_core::ecma::visit::Fold;

pub type Define = HashMap<String, String>;
pub type RawDefine = Define;

pub fn define(
  opts: &Define,
  diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
) -> impl Fold {
  let cm: Arc<swc_core::common::SourceMap> = Default::default();
  let defs = opts
    .iter()
    .filter_map(|(target, replacement)| {
      let mut parse_expr = |text: &String| {
        let fm = cm.new_source_file(FileName::Anon, text.clone());
        let result = parse_file_as_expr(
          &fm,
          Syntax::Es(EsConfig::default()),
          EsVersion::EsNext,
          None,
          &mut vec![],
        );

        if let Err(err) = &result {
          let span = err.span();

          // Push the error to diagnostics
          diagnostics.push(Box::new(
            TraceableError::from_source_file(
              &fm,
              span.lo.0.saturating_sub(1) as usize,
              span.hi.0.saturating_sub(1) as usize,
              "DefinePlugin warning".into(),
              format!("failed to parse {:?}", text),
            )
            .with_help(Some(
              "Consider wrapping it with `JSON.stringify(...)` if a string is expected.",
            ))
            .with_severity(Severity::Warning),
          ));
        }

        result
      };

      let target = parse_expr(target);
      let replacement = parse_expr(replacement);

      if let (Ok(target), Ok(replacement)) = (target, replacement) {
        Some((NodeIgnoringSpan::owned(*target), *replacement))
      } else {
        None
      }
    })
    .collect::<AHashMap<_, _>>();

  inline_globals2(
    Default::default(),
    Default::default(),
    Arc::new(defs),
    Default::default(),
  )
}

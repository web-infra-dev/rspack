use rspack_core::{clean_regexp_in_context_module, try_convert_str_to_context_mode};
use rspack_core::{ContextMode, ContextOptions, DependencyCategory, SpanExt};
use rspack_regex::{regexp_as_str, RspackRegex};
use swc_core::ecma::ast::CallExpr;

use super::JavascriptParserPlugin;
use crate::dependency::RequireContextDependency;
use crate::visitors::expr_matcher::is_require_context;
use crate::visitors::JavascriptParser;

pub struct HarmonyTopLevelThisParserPlugin;

impl JavascriptParserPlugin for HarmonyTopLevelThisParserPlugin {}

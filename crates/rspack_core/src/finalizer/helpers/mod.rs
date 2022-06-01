use once_cell::sync::Lazy;

use rspack_swc::{
  swc_common::{FileName, FilePathMapping, SourceMap},
  swc_ecma_ast::ModuleItem,
  swc_ecma_parser::parse_file_as_module,
  swc_ecma_utils::drop_span,
};

fn parse(code: &str, name: &str) -> Vec<ModuleItem> {
  let cm = SourceMap::new(FilePathMapping::empty());
  let fm = cm.new_source_file(FileName::Custom(name.into()), code.into());
  parse_file_as_module(
    &fm,
    Default::default(),
    Default::default(),
    None,
    &mut vec![],
  )
  .map(|script| drop_span(script.body))
  .map_err(|_| {})
  .unwrap()
}

// struct Helpers;

macro_rules! define {
  (
     $($name:ident, $func:ident)*
  ) => {
    $(
        pub fn $func(to: &mut Vec<ModuleItem>) {
            static STMTS: Lazy<Vec<ModuleItem>> = Lazy::new(|| {
                parse(include_str!(concat!("_rs_", stringify!($name), ".js")), stringify!($name))
            });

            to.extend((*STMTS).clone());
        }
    )*
  };
}

define!(mark_as_esm, mark_as_esm);
define!(define_export, define_export);
define!(get_default_export, get_default_export);
define!(has_own_property, has_own_property);
define!(cjs_runtime_browser, cjs_runtime_browser);
define!(cjs_runtime_node, cjs_runtime_node);

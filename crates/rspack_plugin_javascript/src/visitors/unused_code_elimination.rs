use swc_core::{
  common::{comments::Comments, sync::Lrc, Globals, SourceMap, GLOBALS},
  ecma::{
    ast::Program,
    minifier::{
      self,
      option::{CompressOptions, ExtraOptions, MinifyOptions},
      timing::Timings,
    },
    transforms::base::resolver,
    visit::Fold,
  },
};

use crate::visitors::clear_mark::clear_mark;

pub fn custom_dce<'a>(
  cm: &'a Lrc<SourceMap>,
  comments: Option<&'a dyn Comments>,
  options: MinifyOptions,
  extra: ExtraOptions,
) -> impl Fold + 'a {
  TreeShaker {
    cm,
    comments,
    options,
    extra,
  }
}

struct TreeShaker<'a> {
  cm: &'a Lrc<SourceMap>,
  comments: Option<&'a dyn Comments>,
  options: MinifyOptions,
  extra: ExtraOptions,
}

impl<'a> Fold for TreeShaker<'a> {
  fn fold_program(&mut self, node: Program) -> Program {
    assert!(GLOBALS.is_set());
    let res = clear_mark().fold_program(node);
    let res =
      resolver(self.extra.unresolved_mark, self.extra.top_level_mark, false).fold_program(res);

    minifier::optimize(
      res,
      self.cm.clone(),
      self.comments,
      None,
      &self.options,
      &self.extra,
    )
  }
}

pub fn unused_compress_option() -> CompressOptions {
  CompressOptions {
    arguments: false,
    arrows: false,
    bools: false,
    bools_as_ints: false,
    collapse_vars: false,
    comparisons: false,
    computed_props: false,
    conditionals: false,
    dead_code: false,
    directives: false,
    drop_console: false,
    drop_debugger: false,
    evaluate: false,
    expr: false,
    hoist_fns: false,
    hoist_props: false,
    hoist_vars: false,
    ie8: false,
    if_return: false,
    inline: 0,
    join_vars: false,
    keep_classnames: false,
    keep_fargs: false,
    keep_fnames: false,
    keep_infinity: false,
    loops: false,
    module: false,
    negate_iife: false,
    props: false,
    reduce_fns: false,
    reduce_vars: false,
    sequences: 1,
    side_effects: false,
    switches: false,
    typeofs: false,
    unsafe_passes: false,
    unsafe_arrows: false,
    unsafe_comps: false,
    unsafe_function: false,
    unsafe_math: false,
    unsafe_methods: false,
    unsafe_proto: false,
    unsafe_regexp: false,
    unsafe_symbols: false,
    unsafe_undefined: false,
    unused: true,
    const_to_let: false,
    pristine_globals: false,
    passes: 3,
    ..Default::default()
  }
}

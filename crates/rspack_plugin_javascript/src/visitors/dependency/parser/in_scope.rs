use std::borrow::Cow;

use swc_core::ecma::ast::Pat;

use super::JavascriptParser;

impl JavascriptParser<'_> {
  pub(super) fn in_block_scope<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let old_definitions = self.definitions;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;

    self.in_tagged_template_tag = false;
    self.definitions = self.definitions_db.create_child(&old_definitions);

    f(self);

    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
  }

  pub(super) fn in_class_scope<'a, I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = Cow<'a, Pat>>,
  {
    let old_definitions = self.definitions;
    let old_in_try = self.in_try;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;

    self.in_try = false;
    self.in_tagged_template_tag = false;
    self.definitions = self.definitions_db.create_child(&old_definitions);

    if has_this {
      self.undefined_variable("this");
    }

    self.enter_patterns(params, |this, ident| {
      this.define_variable(ident.sym.to_string());
    });

    f(self);

    self.in_try = old_in_try;
    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
  }

  pub(super) fn in_function_scope<'a, I, F>(&mut self, has_this: bool, params: I, f: F)
  where
    F: FnOnce(&mut Self),
    I: Iterator<Item = Cow<'a, Pat>>,
  {
    let old_definitions = self.definitions;
    let old_top_level_scope = self.top_level_scope;
    let old_in_tagged_template_tag = self.in_tagged_template_tag;

    self.definitions = self.definitions_db.create_child(&old_definitions);
    self.in_tagged_template_tag = false;
    if has_this {
      self.undefined_variable("this");
    }
    self.enter_patterns(params, |this, ident| {
      this.define_variable(ident.sym.to_string());
    });
    f(self);

    self.definitions = old_definitions;
    self.top_level_scope = old_top_level_scope;
    self.in_tagged_template_tag = old_in_tagged_template_tag;
  }
}

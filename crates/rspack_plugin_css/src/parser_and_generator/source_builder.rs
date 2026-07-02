use cow_utils::CowUtils;
use rspack_core::{
  Context, CssLayer, CssModuleRenderCondition,
  rspack_sources::{
    BoxSource, ConcatSource, MapOptions, ObjectPool, RawStringSource, ReplaceSource, Source,
    SourceExt,
  },
};
use rspack_util::{base64::encode_to_string, identifier::make_paths_relative};

const CSS_UTF8_CHARSET: &str = r#"@charset "UTF-8";"#;

pub(crate) struct CssSourceBuilder {
  source: ConcatSource,
  has_charset: bool,
  include_sources_content: bool,
  source_map_context: Context,
}

impl Default for CssSourceBuilder {
  fn default() -> Self {
    Self::new(true, true, Default::default())
  }
}

impl CssSourceBuilder {
  pub(crate) fn new(
    with_charset: bool,
    include_sources_content: bool,
    source_map_context: Context,
  ) -> Self {
    Self {
      source: ConcatSource::default(),
      has_charset: with_charset,
      include_sources_content,
      source_map_context,
    }
  }

  pub(crate) fn push_css_source<'a>(
    &mut self,
    source: BoxSource,
    conditions: impl IntoIterator<Item = &'a CssModuleRenderCondition>,
    trim_source_start: bool,
  ) -> bool {
    let Some(source) = Self::prepare_source(source, trim_source_start) else {
      return false;
    };

    let mut depth = 0;
    // TODO: use PrefixSource to create indent
    for conditions in conditions {
      if let Some(media) = &conditions.media {
        self.add(RawStringSource::from_static("@media "));
        self.add(RawStringSource::from(media.to_string()));
        self.add(RawStringSource::from_static("{\n"));
        depth += 1;
      }

      if let Some(supports) = &conditions.supports {
        self.add(RawStringSource::from_static("@supports ("));
        self.add(RawStringSource::from(supports.to_string()));
        self.add(RawStringSource::from_static(") {\n"));
        depth += 1;
      }

      if let Some(layer) = &conditions.layer {
        match layer {
          CssLayer::Named(layer) => {
            self.add(RawStringSource::from_static("@layer "));
            self.add(RawStringSource::from(layer.to_string()));
            self.add(RawStringSource::from_static(" {\n"));
          }
          CssLayer::Anonymous => {
            self.add(RawStringSource::from_static("@layer {\n"));
          }
        }
        depth += 1;
      }
    }

    // TODO: use PrefixSource to create indent
    self.add(source);
    while depth > 0 {
      depth -= 1;
      self.add(RawStringSource::from_static("\n"));
      self.add(RawStringSource::from_static("}"));
    }
    true
  }

  pub(crate) fn push_line(&mut self) {
    self.add(RawStringSource::from_static("\n"));
  }

  pub(crate) fn set_has_charset(&mut self) {
    self.has_charset = true;
  }

  pub(crate) fn into_source(self) -> BoxSource {
    if self.has_charset {
      ConcatSource::new([
        RawStringSource::from_static(CSS_UTF8_CHARSET).boxed(),
        RawStringSource::from_static("\n").boxed(),
        self.source.boxed(),
      ])
      .boxed()
    } else {
      self.source.boxed()
    }
  }

  pub(crate) fn into_css_text(self) -> String {
    let include_sources_content = self.include_sources_content;
    let source_map_context = self.source_map_context.clone();
    let source = self.into_source();
    let mut css_text = source
      .source()
      .into_string_lossy()
      .cow_replace(crate::utils::AUTO_PUBLIC_PATH_PLACEHOLDER, "")
      .into_owned();

    if let Some(mut source_map) = source.map(&ObjectPool::default(), &MapOptions::default()) {
      if !source_map_context.as_str().is_empty() {
        source_map.set_sources(
          source_map
            .sources()
            .iter()
            .map(|source| make_paths_relative(source_map_context.as_str(), source))
            .collect::<Vec<_>>(),
        );
      }
      if !include_sources_content {
        source_map.set_sources_content(Vec::new());
      }
      let base64_map = encode_to_string(source_map.to_json().as_bytes());
      if !css_text.ends_with('\n') {
        css_text.push('\n');
      }
      css_text.push_str("/*# sourceMappingURL=data:application/json;charset=utf-8;base64,");
      css_text.push_str(&base64_map);
      css_text.push_str("*/");
    }

    css_text
  }

  fn add<S: Source + 'static>(&mut self, source: S) {
    self.source.add(source);
  }

  fn prepare_source(source: BoxSource, trim_source_start: bool) -> Option<BoxSource> {
    if !trim_source_start {
      return Some(source);
    }

    let source_text = source.source().into_string_lossy();
    let source_len = source_text.chars().map(char::len_utf16).sum::<usize>() as u32;
    let leading_len = source_text
      .chars()
      .take_while(|ch| ch.is_whitespace())
      .map(char::len_utf16)
      .sum::<usize>() as u32;
    drop(source_text);

    if leading_len == source_len {
      return None;
    }

    if leading_len == 0 {
      return Some(source);
    }

    let mut source = ReplaceSource::new(source);
    source.replace(0, leading_len, String::new(), None);
    Some(source.boxed())
  }
}

#[cfg(test)]
mod tests {
  use concat_string::concat_string;
  use rspack_core::rspack_sources::{RawStringSource, Source, SourceExt};

  use super::*;

  fn css_source(source: &str) -> BoxSource {
    RawStringSource::from(source.to_string()).boxed()
  }

  fn source_text(source: BoxSource) -> String {
    source.source().into_string_lossy().into_owned()
  }

  fn css_import_conditions(source: &str) -> Vec<CssModuleRenderCondition> {
    let (deps, warnings) =
      css_module_lexer::collect_dependencies(source, css_module_lexer::Mode::Css);
    assert!(warnings.is_empty());

    deps
      .into_iter()
      .filter_map(|dep| match dep {
        css_module_lexer::Dependency::Import {
          media,
          supports,
          layer,
          ..
        } => Some(CssModuleRenderCondition::new(
          media.map(|media| media.trim().into()),
          supports.map(|supports| supports.trim().into()),
          layer.map(|layer| {
            let layer = layer.trim();
            if layer.is_empty() {
              CssLayer::Anonymous
            } else {
              CssLayer::Named(layer.into())
            }
          }),
        )),
        _ => None,
      })
      .collect()
  }

  #[test]
  fn css_source_builder_adds_charset_once() {
    let mut builder = CssSourceBuilder::new(true, true, Default::default());

    builder.push_css_source(css_source(".a{}"), &[], false);

    assert_eq!(
      source_text(builder.into_source()),
      r#"@charset "UTF-8";
.a{}"#
    );
  }

  #[test]
  fn css_source_builder_can_skip_charset() {
    let mut builder = CssSourceBuilder::new(false, true, Default::default());

    builder.push_css_source(css_source(".a{}"), &[], false);

    assert_eq!(source_text(builder.into_source()), ".a{}");
  }

  #[test]
  fn css_source_builder_wraps_css_import_conditions() {
    let conditions = css_import_conditions(
      r#"@import url("./a.css") layer(theme) supports(display: grid) screen;"#,
    );
    let mut builder = CssSourceBuilder::new(false, true, Default::default());

    builder.push_css_source(css_source(".a{}"), &conditions, false);

    assert_eq!(
      source_text(builder.into_source()),
      r#"@media screen{
@supports (display: grid) {
@layer theme {
.a{}
}
}
}"#
    );
  }

  #[test]
  fn css_source_builder_wraps_multiline_css_import_conditions() {
    let conditions = css_import_conditions(
      r#"@import url("./a.css") layer(theme) supports(display: grid) screen;"#,
    );
    let mut builder = CssSourceBuilder::new(false, true, Default::default());

    builder.push_css_source(
      css_source(
        r#".a {
  color: red;
}"#,
      ),
      &conditions,
      false,
    );

    assert_eq!(
      source_text(builder.into_source()),
      r#"@media screen{
@supports (display: grid) {
@layer theme {
.a {
  color: red;
}
}
}
}"#
    );
  }

  #[test]
  fn css_source_builder_wraps_multiple_import_conditions_in_rspack_order() {
    let outer_import =
      css_import_conditions(r#"@import url("./nested.css") screen and (min-width: 768px);"#);
    let inner_import =
      css_import_conditions(r#"@import url("./a.css") layer(theme) supports(display: grid);"#);
    let conditions = outer_import
      .into_iter()
      .chain(inner_import)
      .collect::<Vec<_>>();
    let mut builder = CssSourceBuilder::new(false, true, Default::default());

    assert_eq!(conditions.len(), 2);
    builder.push_css_source(css_source(".a{}"), &conditions, false);

    assert_eq!(
      source_text(builder.into_source()),
      r#"@media screen and (min-width: 768px){
@supports (display: grid) {
@layer theme {
.a{}
}
}
}"#
    );
  }

  #[test]
  fn css_source_builder_pushes_lines_explicitly() {
    let mut builder = CssSourceBuilder::new(false, true, Default::default());

    builder.push_css_source(css_source(".a{}"), &[], false);
    builder.push_line();
    builder.push_css_source(css_source(".b{}"), &[], false);

    assert_eq!(
      source_text(builder.into_source()),
      r#".a{}
.b{}"#
    );
  }

  #[test]
  fn css_source_builder_css_text_removes_auto_public_path_placeholder() {
    let mut builder = CssSourceBuilder::new(true, true, Default::default());

    builder.push_css_source(
      css_source(&concat_string!(
        ".a{background:url(",
        crate::utils::AUTO_PUBLIC_PATH_PLACEHOLDER,
        ");}"
      )),
      &[],
      false,
    );

    assert_eq!(
      builder.into_css_text(),
      r#"@charset "UTF-8";
.a{background:url();}"#
    );
  }
}

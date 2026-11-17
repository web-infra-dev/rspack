use rspack_core::{
  Context, CssLayer, CssModuleRenderCondition,
  rspack_sources::{
    BoxSource, ConcatSource, MapOptions, RawStringSource, ReplaceSource, Source, SourceExt,
  },
};
use rspack_util::{
  base64::encode_to_string, identifier::make_paths_relative, source_map::SourceMapKind,
};

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

  pub(crate) fn into_css_text(self, source_map_kind: SourceMapKind) -> String {
    let include_sources_content = self.include_sources_content;
    let source_map_context = self.source_map_context.clone();
    let source = self.into_source();
    let mut css_text = crate::utils::AUTO_PUBLIC_PATH_MATCHER
      .replace(&source.source().into_string_lossy(), "")
      .into_owned();

    // CSS embedded in JavaScript must not expose maps for external or hidden devtool modes.
    if source_map_kind.contains(SourceMapKind::Inline)
      && let Some(mut source_map) = source.map(&MapOptions::default())
    {
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

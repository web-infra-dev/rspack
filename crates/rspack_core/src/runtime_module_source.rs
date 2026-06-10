use rspack_collections::Identifier;
use rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt};

pub fn render_runtime_module_source(
  identifier: Identifier,
  source: BoxSource,
  should_isolate: bool,
  supports_arrow_function: bool,
) -> BoxSource {
  let mut sources = ConcatSource::default();
  if source.size() == 0 {
    return sources.boxed();
  }

  sources.add(RawStringSource::from(format!("// {identifier}\n")));
  if should_isolate {
    sources.add(RawStringSource::from_static(if supports_arrow_function {
      "(() => {\n"
    } else {
      "!function() {\n"
    }));
  }
  sources.add(source);
  if should_isolate {
    sources.add(RawStringSource::from_static(if supports_arrow_function {
      "\n})();\n"
    } else {
      "\n}();\n"
    }));
  }

  sources.boxed()
}

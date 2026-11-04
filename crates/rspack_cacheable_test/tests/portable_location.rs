use std::path::{Path, PathBuf};

use camino::Utf8PathBuf;
use rspack_cacheable::CacheableContext;
use rspack_location::{
  DependencyLocation, PortableDependencyLocation, PortableLocationShift, RealDependencyLocation,
  SourcePosition, SyntheticDependencyLocation, portable::quote_ascii,
};

struct Context(PathBuf);

impl CacheableContext for Context {
  fn project_root(&self) -> Option<&Path> {
    Some(&self.0)
  }
}

fn position(source: &str, offset: usize) -> SourcePosition {
  let prefix = &source[..offset];
  SourcePosition {
    line: prefix.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1,
    column: prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32 + 1,
  }
}

fn location(source: &str, start: usize, end: usize) -> RealDependencyLocation {
  RealDependencyLocation::new(
    position(source, start),
    (start != end).then(|| position(source, end)),
  )
}

fn fixture(context: &Context) -> (Vec<DependencyLocation>, Vec<DependencyLocation>) {
  let names = ["src/读者-😀.js", "second.js", "third.js"];
  let canonical_literals = names.map(|name| quote_ascii(&format!("[portable-root]/{name}")));
  let paths = names.map(|name| Utf8PathBuf::from_path_buf(context.0.join(name)).unwrap());
  let literals = paths.each_ref().map(|path| quote_ascii(path.as_str()));
  let source = |literals: &[String; 3]| {
    format!(
      "/* 😀é */ const p = {}; {}; import 'dep';\n{}; import 'other';\nlast\n",
      literals[0], literals[1], literals[2],
    )
  };
  let canonical = source(&canonical_literals);
  let rendered = source(&literals);
  let canonical_starts = canonical_literals
    .each_ref()
    .map(|literal| canonical.find(literal).unwrap());
  let rendered_starts = literals
    .each_ref()
    .map(|literal| rendered.find(literal).unwrap());
  let shifts = (0..3)
    .map(|index| PortableLocationShift {
      start: position(&canonical, canonical_starts[index]),
      canonical_width: canonical_literals[index].len() as u32,
      path: paths[index].clone(),
    })
    .collect::<Vec<_>>();

  let ranges = |text: &str, starts: &[usize; 3], literals: &[String; 3]| {
    let first_end = starts[0] + literals[0].len();
    let second_end = starts[1] + literals[1].len();
    vec![
      (starts[0], starts[0]),  // A point exactly at the replacement's start.
      (first_end, first_end),  // A point exactly at its half-open end.
      (starts[0], first_end),  // The full literal, ending exactly at the boundary.
      (first_end, starts[1]),  // Gap whose endpoints depend on the first width only.
      (starts[1], second_end), // Both preceding and enclosed path widths.
      (
        text.find("import 'dep'").unwrap(),
        text.find(";\n").unwrap() + 1,
      ),
      (
        starts[0],
        text.find("import 'other'").unwrap() + "import 'other'".len(),
      ),
      (text.find("last").unwrap(), text.len()), // Earlier lines never affect columns.
    ]
  };
  let actual = ranges(&canonical, &canonical_starts, &canonical_literals)
    .into_iter()
    .map(|(start, end)| {
      PortableDependencyLocation::from_canonical(location(&canonical, start, end), &shifts)
    })
    .collect::<Vec<_>>();
  let expected = ranges(&rendered, &rendered_starts, &literals)
    .into_iter()
    .map(|(start, end)| DependencyLocation::Real(location(&rendered, start, end)))
    .collect::<Vec<_>>();
  assert!(matches!(actual[0], DependencyLocation::Real(_)));
  assert!(matches!(actual[1], DependencyLocation::Portable(_)));
  assert!(matches!(actual[7], DependencyLocation::Real(_)));

  // Interior endpoints snap outward like SourceView; an interior insertion stays empty.
  let start = canonical_starts[0];
  let point =
    PortableDependencyLocation::from_canonical(location(&canonical, start + 1, start + 1), &shifts);
  assert_eq!(point, expected[0]);
  let span =
    PortableDependencyLocation::from_canonical(location(&canonical, start + 1, start + 2), &shifts);
  assert_eq!(span, expected[2]);
  (actual, expected)
}

#[test]
fn test_portable_location_rebases_columns_and_serializes_canonically() {
  let temp = std::env::temp_dir();
  let writer = Context(temp.join("writer-读者-😀").join("project"));
  let (original, expected) = fixture(&writer);
  assert_eq!(original, expected);
  let (bytes, bound) = rspack_cacheable::to_bytes_with_binding(&original, &writer).unwrap();
  assert!(!bound);
  let bailouts = original
    .iter()
    .cloned()
    .map(|loc| rspack_core::OptimizationBailoutItem::SideEffects {
      node_type: "Statement".into(),
      loc,
      short_id: "./module.js".into(),
    })
    .collect::<Vec<_>>();
  let bailout_bytes = rspack_cacheable::to_bytes(&bailouts, &writer).unwrap();
  for root in [
    writer.0.clone(),
    temp.join("r"),
    temp.join("longer-根-😀-\"-root").join("project"),
  ] {
    let reader = Context(root);
    let restored: Vec<DependencyLocation> = rspack_cacheable::from_bytes(&bytes, &reader).unwrap();
    let (cold, expected) = fixture(&reader);
    assert_eq!(restored, expected);
    assert_eq!(
      restored.iter().map(ToString::to_string).collect::<Vec<_>>(),
      expected.iter().map(ToString::to_string).collect::<Vec<_>>()
    );
    let (cold_bytes, bound) = rspack_cacheable::to_bytes_with_binding(&cold, &reader).unwrap();
    assert!(!bound);
    assert_eq!(bytes, cold_bytes);
    let restored_bailouts: Vec<rspack_core::OptimizationBailoutItem> =
      rspack_cacheable::from_bytes(&bailout_bytes, &reader).unwrap();
    assert_eq!(
      restored_bailouts
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>(),
      expected
        .iter()
        .map(|loc| format!("Statement with side_effects in source code at ./module.js:{loc}"))
        .collect::<Vec<_>>()
    );
    // Equal real positions order identically, regardless of the stored representation.
    for (left, expected_left) in restored.iter().zip(&expected) {
      for (right, expected_right) in restored.iter().zip(&expected) {
        assert_eq!(left.cmp(right), expected_left.cmp(expected_right));
      }
      assert!(left < &DependencyLocation::Synthetic(SyntheticDependencyLocation::new("generated")));
    }
  }
}

#[test]
fn test_portable_path_quote_uses_ascii_utf16_escapes() {
  assert_eq!(
    quote_ascii("\"\\\n\r\t\u{8}\u{c}\0é😀"),
    r#""\"\\\n\r\t\b\f\u0000\u00e9\ud83d\ude00""#
  );
}

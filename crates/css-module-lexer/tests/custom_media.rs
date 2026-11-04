use css_module_lexer::{
  CustomMediaDefinitionValue, Dependency, MediaQueryInstruction, Mode, collect_dependencies,
};

#[test]
fn emits_custom_media_dependencies_with_minimal_fold_programs() {
  let input = r#"
    @media (--wide) and (hover) { .wide {} }
    @custom-media --wide (width > 40rem);
  "#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);

  assert!(warnings.is_empty());

  let media = dependencies
    .iter()
    .find_map(|dependency| match dependency {
      Dependency::Media { query } => Some(dependencies.media_query(*query)),
      _ => None,
    })
    .expect("the @media prelude should produce a dependency");
  assert_eq!(media.queries.len(), 1);
  assert!(matches!(
    media.queries[0].instructions.as_slice(),
    [
      MediaQueryInstruction::Reference(_),
      MediaQueryInstruction::Opaque(_),
      MediaQueryInstruction::And(2),
    ]
  ));

  let definition = dependencies
    .iter()
    .find_map(|dependency| match dependency {
      Dependency::CustomMedia {
        definition: Some(definition),
        ..
      } => Some(dependencies.custom_media_definition(*definition)),
      _ => None,
    })
    .expect("the @custom-media rule should produce a dependency");
  assert_eq!(definition.name, "--wide");
  assert!(matches!(
    definition.value,
    CustomMediaDefinitionValue::Condition(_)
  ));
}

#[test]
fn collects_custom_media_dependencies_without_parser_policy() {
  let input = "@custom-media --wide (width > 40rem); @media (--wide) {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);

  assert!(warnings.is_empty());
  assert!(
    dependencies
      .iter()
      .any(|dependency| matches!(dependency, Dependency::CustomMedia { .. }))
  );
  assert!(
    dependencies
      .iter()
      .any(|dependency| matches!(dependency, Dependency::Media { .. }))
  );
}

#[test]
fn keeps_the_main_dependency_scan_intact() {
  let input = r#"
    @custom-media --wide (width > 40rem);
    @media (--wide) { .wide {} }
  "#;
  let (dependencies, warnings) = collect_dependencies(input, Mode::Local);

  assert!(warnings.is_empty());
  assert!(
    dependencies
      .iter()
      .any(|dependency| matches!(dependency, Dependency::LocalClass { name: ".wide", .. }))
  );
}

#[test]
fn leaves_block_bearing_custom_media_as_written() {
  let input = "@custom-media --invalid (width > 1px) {}";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);

  assert!(warnings.is_empty());
  assert!(dependencies.iter().all(|dependency| !matches!(
    dependency,
    Dependency::CustomMedia { .. } | Dependency::Replace { .. }
  )));
}

#[test]
fn emits_one_removal_dependency_for_an_invalid_definition() {
  let input = "@custom-media (width > 1px);";
  let (dependencies, warnings) = collect_dependencies(input, Mode::Css);

  assert!(warnings.is_empty());
  assert!(matches!(
    dependencies.dependencies(),
    [Dependency::CustomMedia {
      definition: None,
      range,
    }] if *range == css_module_lexer::Range::new(0, input.len() as u32)
  ));
}

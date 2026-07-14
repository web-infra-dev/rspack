use rspack::builder::Builder as _;
use rspack_core::Compiler;
use rspack_paths::ArcPath;
use rspack_tasks::within_compiler_context_for_testing_sync;

#[test]
fn compilation_level_dependencies_stay_in_their_own_added_iterators() {
  within_compiler_context_for_testing_sync(|| {
    let mut compiler = Compiler::builder().build().expect("compiler should build");
    let compilation = &mut compiler.compilation;
    let file = ArcPath::from("/project/file.tsx");
    let context = ArcPath::from("/project/context");
    let missing = ArcPath::from("/project/missing.tsx");
    let build = ArcPath::from("/project/build.config.js");

    compilation.file_dependencies.insert(file.clone());
    compilation.context_dependencies.insert(context.clone());
    compilation.missing_dependencies.insert(missing.clone());
    compilation.build_dependencies.insert(build.clone());

    let (_, file_added, _, _) = compilation.file_dependencies();
    let (_, context_added, _, _) = compilation.context_dependencies();
    let (_, missing_added, _, _) = compilation.missing_dependencies();
    let (_, build_added, _, _) = compilation.build_dependencies();

    assert_eq!(file_added.cloned().collect::<Vec<_>>(), [file]);
    assert_eq!(context_added.cloned().collect::<Vec<_>>(), [context]);
    assert_eq!(missing_added.cloned().collect::<Vec<_>>(), [missing]);
    assert_eq!(build_added.cloned().collect::<Vec<_>>(), [build]);
  });
}

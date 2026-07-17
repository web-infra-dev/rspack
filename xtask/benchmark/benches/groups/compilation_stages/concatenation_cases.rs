#![allow(clippy::unwrap_used)]

use rspack_fs::{MemoryFileSystem, WritableFileSystem};

const CONCAT_GROUPS: usize = 160;
const CONCAT_MODULES_PER_GROUP: usize = 12;
const CONCAT_SHARED_ROOTS: usize = 192;
const CONCAT_SHARED_MODULES: usize = 128;
const CONCAT_SHARED_WINDOW: usize = 16;
const CONCAT_LOCAL_MODULES_PER_ROOT: usize = 8;
const CONCAT_LAZY_PANELS_PER_ROOT: usize = 3;

#[derive(Clone, Copy)]
enum ConcatenationBenchmarkTopology {
  DisjointGroups,
  SharedRoots(SharedRootsTopology),
}

#[derive(Clone, Copy)]
enum SharedRootsTopology {
  Standard,
  Bailouts,
  UnsupportedSyntax,
}

#[derive(Clone, Copy)]
pub(super) enum ConcatenationStatistic {
  IncorrectChunksOfImporter,
  IncorrectModuleDependency,
  ImporterFailed,
}

impl ConcatenationStatistic {
  pub(super) const fn log_label(self) -> &'static str {
    match self {
      Self::IncorrectChunksOfImporter => "incorrect chunks of importer",
      Self::IncorrectModuleDependency => "incorrect module dependency",
      Self::ImporterFailed => "importer failed",
    }
  }
}

#[derive(Clone, Copy)]
pub(super) struct ConcatenationBenchmarkCase {
  pub(super) name: &'static str,
  pub(super) setup_label: &'static str,
  pub(super) expected_statistics: &'static [ConcatenationStatistic],
  topology: ConcatenationBenchmarkTopology,
}

pub(super) const CONCATENATION_BENCHMARK_CASES: [ConcatenationBenchmarkCase; 4] = [
  ConcatenationBenchmarkCase {
    name: "rust@create_concatenate_module",
    setup_label: "create_concatenate_module setup",
    expected_statistics: &[],
    topology: ConcatenationBenchmarkTopology::DisjointGroups,
  },
  ConcatenationBenchmarkCase {
    name: "rust@create_concatenate_module_shared_roots",
    setup_label: "create_concatenate_module_shared_roots setup",
    expected_statistics: &[ConcatenationStatistic::IncorrectChunksOfImporter],
    topology: ConcatenationBenchmarkTopology::SharedRoots(SharedRootsTopology::Standard),
  },
  ConcatenationBenchmarkCase {
    name: "rust@create_concatenate_module_bailouts",
    setup_label: "create_concatenate_module_bailouts setup",
    expected_statistics: &[
      ConcatenationStatistic::IncorrectChunksOfImporter,
      ConcatenationStatistic::ImporterFailed,
    ],
    topology: ConcatenationBenchmarkTopology::SharedRoots(SharedRootsTopology::Bailouts),
  },
  ConcatenationBenchmarkCase {
    name: "rust@create_concatenate_module_unsupported_syntax",
    setup_label: "create_concatenate_module_unsupported_syntax setup",
    expected_statistics: &[ConcatenationStatistic::IncorrectModuleDependency],
    topology: ConcatenationBenchmarkTopology::SharedRoots(SharedRootsTopology::UnsupportedSyntax),
  },
];

pub(super) async fn prepare_concatenation_benchmark_case(
  case: ConcatenationBenchmarkCase,
  fs: &MemoryFileSystem,
) {
  match case.topology {
    ConcatenationBenchmarkTopology::DisjointGroups => {
      prepare_large_concatenation_case(CONCAT_GROUPS, CONCAT_MODULES_PER_GROUP, fs).await;
    }
    ConcatenationBenchmarkTopology::SharedRoots(topology) => {
      prepare_shared_concatenation_case(topology, fs).await;
    }
  }
}

pub(super) async fn prepare_default_concatenation_case(fs: &MemoryFileSystem) {
  prepare_large_concatenation_case(CONCAT_GROUPS, CONCAT_MODULES_PER_GROUP, fs).await;
}

async fn prepare_large_concatenation_case(
  groups: usize,
  modules_per_group: usize,
  fs: &MemoryFileSystem,
) {
  fs.create_dir_all("/src".into()).await.unwrap();
  let mut root_imports = Vec::with_capacity(groups);
  let mut root_values = Vec::with_capacity(groups);

  for group in 0..groups {
    let group_dir = format!("/src/group-{group}");
    fs.create_dir_all(group_dir.as_str().into()).await.unwrap();

    let mut group_imports = Vec::with_capacity(modules_per_group);
    let mut group_values = Vec::with_capacity(modules_per_group);

    for module in 0..modules_per_group {
      let file = format!("/src/group-{group}/module-{module}.js");
      let code = if module == 0 {
        format!("export const value = {group};")
      } else {
        format!(
          "import {{ value as prev }} from './module-{}.js'; export const value = prev + {};",
          module - 1,
          module
        )
      };
      fs.write(file.as_str().into(), code.as_bytes())
        .await
        .unwrap();
      group_imports.push(format!(
        "import {{ value as v{module} }} from './module-{module}.js';"
      ));
      group_values.push(format!("v{module}"));
    }

    let group_entry = format!(
      "{}\nexport default {};",
      group_imports.join("\n"),
      group_values.join(" + ")
    );
    fs.write(
      format!("/src/group-{group}/entry.js").as_str().into(),
      group_entry.as_bytes(),
    )
    .await
    .unwrap();

    root_imports.push(format!(
      "import g{group} from '/src/group-{group}/entry.js';"
    ));
    root_values.push(format!("g{group}"));
  }

  let entry = format!(
    "{}\nconsole.log({});",
    root_imports.join("\n"),
    root_values.join(" + ")
  );
  fs.write("/src/index.js".into(), entry.as_bytes())
    .await
    .unwrap();
}

async fn prepare_shared_concatenation_case(topology: SharedRootsTopology, fs: &MemoryFileSystem) {
  fs.create_dir_all("/src/shared".into()).await.unwrap();
  fs.create_dir_all("/src/routes".into()).await.unwrap();

  for module in 0..CONCAT_SHARED_MODULES {
    let source = format!("export default {module};");
    fs.write(
      format!("/src/shared/shared-{module}.js").as_str().into(),
      source.as_bytes(),
    )
    .await
    .unwrap();
  }

  let mut route_imports = Vec::with_capacity(CONCAT_SHARED_ROOTS);
  for root in 0..CONCAT_SHARED_ROOTS {
    let route_dir = format!("/src/routes/route-{root}");
    fs.create_dir_all(route_dir.as_str().into()).await.unwrap();

    for module in 0..CONCAT_LOCAL_MODULES_PER_ROOT {
      let source = if module == 0 {
        format!("export default {root};")
      } else {
        format!(
          "import value from './local-{}.js'; export default value + {};",
          module - 1,
          module
        )
      };
      fs.write(
        format!("{route_dir}/local-{module}.js").as_str().into(),
        source.as_bytes(),
      )
      .await
      .unwrap();
    }

    let lazy_panel_imports = match topology {
      SharedRootsTopology::Bailouts => {
        let blocker_source = format!(
          "import value from './local-{}.js'; eval(''); console.log(value);",
          CONCAT_LOCAL_MODULES_PER_ROOT - 1
        );
        fs.write(
          format!("{route_dir}/blocker.js").as_str().into(),
          blocker_source.as_bytes(),
        )
        .await
        .unwrap();

        let mut imports = Vec::with_capacity(CONCAT_LAZY_PANELS_PER_ROOT);
        for panel in 0..CONCAT_LAZY_PANELS_PER_ROOT {
          let left = panel % CONCAT_LOCAL_MODULES_PER_ROOT;
          let right = (panel + 1) % CONCAT_LOCAL_MODULES_PER_ROOT;
          let source = format!(
            "import left from './local-{left}.js'; import right from './local-{right}.js'; export default left + right;"
          );
          fs.write(
            format!("{route_dir}/panel-{panel}.js").as_str().into(),
            source.as_bytes(),
          )
          .await
          .unwrap();
          imports.push(format!("import('./panel-{panel}.js')"));
        }
        imports
      }
      SharedRootsTopology::Standard | SharedRootsTopology::UnsupportedSyntax => Vec::new(),
    };

    let mut imports = vec![format!(
      "import local from './local-{}.js';",
      CONCAT_LOCAL_MODULES_PER_ROOT - 1
    )];
    if matches!(topology, SharedRootsTopology::Bailouts) {
      imports.push("import './blocker.js';".to_string());
    }
    if matches!(topology, SharedRootsTopology::UnsupportedSyntax) {
      imports.push("require('./local-0.js');".to_string());
    }
    let mut values = vec!["local".to_string()];
    for offset in 0..CONCAT_SHARED_WINDOW {
      let module = (root + offset) % CONCAT_SHARED_MODULES;
      imports.push(format!(
        "import shared_{module} from '/src/shared/shared-{module}.js';"
      ));
      values.push(format!("shared_{module}"));
    }
    let source = match topology {
      SharedRootsTopology::Bailouts => format!(
        "{}\nexport const panels = Promise.all([{}]);\nexport default {};",
        imports.join("\n"),
        lazy_panel_imports.join(", "),
        values.join(" + ")
      ),
      SharedRootsTopology::Standard | SharedRootsTopology::UnsupportedSyntax => format!(
        "{}\nexport default {};",
        imports.join("\n"),
        values.join(" + ")
      ),
    };
    fs.write(
      format!("{route_dir}/entry.js").as_str().into(),
      source.as_bytes(),
    )
    .await
    .unwrap();
    route_imports.push(format!("import('/src/routes/route-{root}/entry.js')"));
  }

  let entry = format!(
    "Promise.all([{}]).then(routes => console.log(routes));",
    route_imports.join(",\n")
  );
  fs.write("/src/index.js".into(), entry.as_bytes())
    .await
    .unwrap();
}

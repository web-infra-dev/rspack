use std::{collections::BTreeMap, sync::Mutex};

use derive_more::Debug;
use rspack_core::{
  ChunkGraph, Compilation, CompilationRecordModules, CompilationReviveModules, LibIdentOptions,
  ModuleId, ModuleIdsArtifact, Plugin,
};
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_hook::{plugin, plugin_hook};
use serde_json::Value;

use crate::id_helpers::ModuleFilterFn;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SyncModuleIdsPluginMode {
  Read,
  Create,
  #[default]
  Merge,
  Update,
}

#[derive(Debug, Clone)]
pub struct SyncModuleIdsPluginOptions {
  pub path: String,
  pub context: Option<String>,
  #[debug(skip)]
  pub test: Option<ModuleFilterFn>,
  pub mode: SyncModuleIdsPluginMode,
}

fn parse_module_ids(buffer: &[u8]) -> Result<BTreeMap<String, ModuleId>> {
  let json: BTreeMap<String, Value> = serde_json::from_slice(buffer).to_rspack_result()?;
  let mut data = BTreeMap::new();
  for (key, value) in json {
    let id = match value {
      Value::String(value) => Some(ModuleId::from(value)),
      Value::Number(value) => Some(ModuleId::from(value.to_string())),
      Value::Null => None,
      _ => {
        return Err(error!(
          "SyncModuleIdsPlugin: Expected module id for '{}' to be a string, number or null.",
          key
        ));
      }
    };
    if let Some(id) = id {
      data.insert(key, id);
    }
  }
  Ok(data)
}

#[plugin]
#[derive(Debug)]
pub struct SyncModuleIdsPlugin {
  path: String,
  context: Option<String>,
  #[debug(skip)]
  test: Option<ModuleFilterFn>,
  mode: SyncModuleIdsPluginMode,
  #[debug(skip)]
  state: Mutex<Option<BTreeMap<String, ModuleId>>>,
}

impl SyncModuleIdsPlugin {
  pub fn new(options: SyncModuleIdsPluginOptions) -> Self {
    Self::new_inner(
      options.path,
      options.context,
      options.test,
      options.mode,
      Default::default(),
    )
  }

  fn need_read(&self) -> bool {
    matches!(
      self.mode,
      SyncModuleIdsPluginMode::Read
        | SyncModuleIdsPluginMode::Merge
        | SyncModuleIdsPluginMode::Update
    )
  }

  fn need_write(&self) -> bool {
    matches!(
      self.mode,
      SyncModuleIdsPluginMode::Create
        | SyncModuleIdsPluginMode::Merge
        | SyncModuleIdsPluginMode::Update
    )
  }

  fn need_prune(&self) -> bool {
    self.mode == SyncModuleIdsPluginMode::Update
  }
}

#[plugin_hook(CompilationReviveModules for SyncModuleIdsPlugin)]
async fn revive_modules(
  &self,
  compilation: &Compilation,
  modules: &rspack_collections::IdentifierSet,
  module_ids: &mut ModuleIdsArtifact,
) -> Result<()> {
  if !self.need_read() {
    return Ok(());
  }

  let path = rspack_paths::Utf8Path::new(&self.path);
  let Some(data) = (match compilation.intermediate_filesystem.read_file(path).await {
    Ok(buffer) => Some(parse_module_ids(&buffer)?),
    Err(rspack_fs::Error::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => None,
    Err(err) => return Err(err.into()),
  }) else {
    let mut state = self
      .state
      .lock()
      .expect("SyncModuleIdsPlugin state should not be poisoned");
    *state = None;
    return Ok(());
  };

  let module_graph = compilation.get_module_graph();
  let mut used_ids = module_ids
    .iter()
    .map(|(module_identifier, id)| (id.clone(), *module_identifier))
    .collect::<BTreeMap<_, _>>();
  let context = self
    .context
    .as_deref()
    .unwrap_or(compilation.options.context.as_str());

  for module_identifier in modules {
    let Some(module) = module_graph.module_by_identifier(module_identifier) else {
      continue;
    };
    if let Some(test) = &self.test
      && !test(compilation.compiler_id(), module.as_ref()).await?
    {
      continue;
    }
    let Some(name) = module.lib_ident(LibIdentOptions { context }) else {
      continue;
    };
    let Some(id) = data.get(name.as_ref()).cloned() else {
      continue;
    };
    if let Some(used_by) = used_ids.get(&id)
      && *used_by != module.identifier()
    {
      return Err(error!(
        "SyncModuleIdsPlugin: Unable to restore id '{}' from '{}' as it's already used.",
        id, self.path
      ));
    }
    if let Some(old_id) = ChunkGraph::get_module_id(module_ids, module.identifier()) {
      used_ids.remove(old_id);
    }
    ChunkGraph::set_module_id(module_ids, module.identifier(), id.clone());
    used_ids.insert(id, module.identifier());
  }

  let mut state = self
    .state
    .lock()
    .expect("SyncModuleIdsPlugin state should not be poisoned");
  *state = Some(data);

  Ok(())
}

#[plugin_hook(CompilationRecordModules for SyncModuleIdsPlugin)]
async fn record_modules(
  &self,
  compilation: &Compilation,
  module_ids: &ModuleIdsArtifact,
) -> Result<()> {
  let old_data = {
    let mut state = self
      .state
      .lock()
      .expect("SyncModuleIdsPlugin state should not be poisoned");
    state.take().unwrap_or_default()
  };
  let mut data = if self.need_prune() {
    BTreeMap::default()
  } else {
    old_data.clone()
  };

  let module_graph = compilation.get_module_graph();
  let context = self
    .context
    .as_deref()
    .unwrap_or(compilation.options.context.as_str());

  for (_, module) in module_graph.modules() {
    if let Some(test) = &self.test
      && !test(compilation.compiler_id(), module.as_ref()).await?
    {
      continue;
    }
    let Some(name) = module.lib_ident(LibIdentOptions { context }) else {
      continue;
    };
    let Some(id) = ChunkGraph::get_module_id(module_ids, module.identifier()).cloned() else {
      continue;
    };
    data.insert(name.into_owned(), id);
  }

  let json = if data != old_data {
    Some(simd_json::to_string(&data).to_rspack_result()?)
  } else {
    None
  };

  {
    let mut state = self
      .state
      .lock()
      .expect("SyncModuleIdsPlugin state should not be poisoned");
    *state = Some(data);
  }

  if let Some(json) = json {
    let path = rspack_paths::Utf8Path::new(&self.path);
    if let Some(dir) = path.parent() {
      compilation
        .intermediate_filesystem
        .create_dir_all(dir)
        .await?;
    }
    compilation
      .intermediate_filesystem
      .write(path, json.as_bytes())
      .await?;
  }

  Ok(())
}

impl Plugin for SyncModuleIdsPlugin {
  fn name(&self) -> &'static str {
    "SyncModuleIdsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    if self.need_read() {
      ctx
        .compilation_hooks
        .revive_modules
        .tap(revive_modules::new(self));
    }

    if self.need_write() {
      ctx
        .compilation_hooks
        .record_modules
        .tap(record_modules::new(self));
    }

    Ok(())
  }
}

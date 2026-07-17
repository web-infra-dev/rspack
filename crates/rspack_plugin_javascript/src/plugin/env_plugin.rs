use std::{borrow::Cow, sync::LazyLock};

use concat_string::concat_string;
use rspack_core::{
  Compilation, CompilationId, CompilationParams, CompilerCompilation, Plugin,
  property_access_with_optional,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::FxDashMap;
use serde_json::{Map, Value};
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{MemberExpr, MemberProp, MetaPropKind};

use crate::{
  parser_plugin::define_plugin::{
    VALUE_DEP_PREFIX,
    utils::{code_object_property_to_string, code_object_to_string, code_to_string, wrap_code},
  },
  visitors::{
    DestructuringAssignmentProperties, ExportedVariableInfo, ExpressionExpressionInfo,
    JavascriptParser,
  },
};

#[plugin]
#[derive(Debug, Default)]
pub struct EnvPlugin;

impl EnvPlugin {
  pub(crate) fn collect(compilation_id: CompilationId, definitions: &ImportMetaEnvDefinitions) {
    let mut state = IMPORT_META_ENV_DEFINITIONS_MAP
      .entry(compilation_id)
      .or_default();
    state.definitions.merge(definitions);
    state.serialized = None;
  }

  fn finalize(compilation: &mut Compilation) {
    let compilation_id = compilation.id();
    let mut state = IMPORT_META_ENV_DEFINITIONS_MAP
      .entry(compilation_id)
      .or_default();
    let serialized = match &state.serialized {
      Some(serialized) => serialized.clone(),
      None => {
        let serialized = serialize_import_meta_env_definitions(&state.definitions, None);
        state.serialized = Some(serialized.clone());
        serialized
      }
    };
    compilation
      .value_cache_versions
      .insert(IMPORT_META_ENV_VALUE_DEP_KEY.to_string(), serialized);
  }
}

#[plugin_hook(CompilerCompilation for EnvPlugin, stage = i32::MAX, tracing=false)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  if compilation.options.experiments.env {
    EnvPlugin::finalize(compilation);
  }
  Ok(())
}

impl Plugin for EnvPlugin {
  fn name(&self) -> &'static str {
    "rspack.EnvPlugin"
  }

  fn clear_cache(&self, id: CompilationId) {
    IMPORT_META_ENV_DEFINITIONS_MAP.remove(&id);
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    Ok(())
  }
}

pub(crate) const IMPORT_META_ENV: &str = "import.meta.env";
pub(crate) const IMPORT_META_ENV_PREFIX: &str = "import.meta.env.";
pub(crate) const IMPORT_META_ENV_VALUE_DEP_KEY: &str = "rspack/DefinePlugin import.meta.env.*";
const TYPEOF_PREFIX: &str = "typeof ";

#[derive(Debug, Default)]
pub(crate) struct ImportMetaEnvDefinitions {
  definitions: Map<String, Value>,
  typeof_definitions: Map<String, Value>,
}

impl ImportMetaEnvDefinitions {
  pub(crate) fn collect(&mut self, name: &str, code: &Value) {
    if let Some(name) = name.strip_prefix(TYPEOF_PREFIX)
      && is_import_meta_env_name(name)
    {
      self
        .typeof_definitions
        .entry(name.to_string())
        .or_insert_with(|| code.clone());
    } else if name == IMPORT_META_ENV {
      self.collect_json_object(code);
    } else if let Some(name) = name.strip_prefix(IMPORT_META_ENV_PREFIX) {
      insert_definition(
        &mut self.definitions,
        &name.split('.').collect::<Vec<_>>(),
        code.clone(),
      );
    }
  }

  fn collect_json_object(&mut self, code: &Value) {
    let object = match code {
      Value::Object(object) => object.clone(),
      Value::String(code) => {
        let Ok(Value::Object(object)) = serde_json::from_str::<Value>(code) else {
          return;
        };
        object
          .into_iter()
          .map(|(key, value)| (key, json_value_to_define_value(value)))
          .collect()
      }
      _ => return,
    };
    merge_definitions(&mut self.definitions, &object);
  }

  fn merge(&mut self, other: &Self) {
    merge_definitions(&mut self.definitions, &other.definitions);
    for (key, value) in &other.typeof_definitions {
      self
        .typeof_definitions
        .entry(key.clone())
        .or_insert_with(|| value.clone());
    }
  }

  fn get(&self, path: &[Atom]) -> Option<&Value> {
    let (first, rest) = path.split_first()?;
    let mut value = self.definitions.get(first.as_ref())?;
    for member in rest {
      value = match value {
        Value::Object(object) => object.get(member.as_ref())?,
        Value::Array(array) => array.get(member.parse::<usize>().ok()?)?,
        _ => return None,
      };
    }
    Some(value)
  }
}

fn insert_definition(object: &mut Map<String, Value>, path: &[&str], value: Value) {
  let Some((key, rest)) = path.split_first() else {
    return;
  };
  if rest.is_empty() {
    object.insert((*key).to_string(), value);
    return;
  }

  let child = object
    .entry((*key).to_string())
    .or_insert_with(|| Value::Object(Map::new()));
  if !child.is_object() {
    *child = Value::Object(Map::new());
  }
  let Value::Object(child) = child else {
    unreachable!("child was initialized as an object")
  };
  insert_definition(child, rest, value);
}

fn merge_definitions(target: &mut Map<String, Value>, source: &Map<String, Value>) {
  for (key, value) in source {
    match (target.get_mut(key), value) {
      (Some(Value::Object(target)), Value::Object(source)) => {
        merge_definitions(target, source);
      }
      (Some(_), _) => {}
      (None, _) => {
        target.insert(key.clone(), value.clone());
      }
    }
  }
}

#[derive(Debug, Default)]
struct ImportMetaEnvDefinitionsState {
  definitions: ImportMetaEnvDefinitions,
  serialized: Option<String>,
}

static IMPORT_META_ENV_DEFINITIONS_MAP: LazyLock<
  FxDashMap<CompilationId, ImportMetaEnvDefinitionsState>,
> = LazyLock::new(Default::default);

fn json_value_to_define_value(value: Value) -> Value {
  match value {
    Value::String(value) => Value::String(serde_json::to_string(&value).expect("string is JSON")),
    Value::Array(array) => {
      Value::Array(array.into_iter().map(json_value_to_define_value).collect())
    }
    Value::Object(object) => Value::Object(
      object
        .into_iter()
        .map(|(key, value)| (key, json_value_to_define_value(value)))
        .collect(),
    ),
    value => value,
  }
}

fn serialize_import_meta_env_definitions(
  definitions: &ImportMetaEnvDefinitions,
  obj_keys: Option<&DestructuringAssignmentProperties>,
) -> String {
  code_object_to_string(&definitions.definitions, None, obj_keys).into_owned()
}

pub(crate) fn import_meta_env_definitions_string(compilation_id: CompilationId) -> String {
  IMPORT_META_ENV_DEFINITIONS_MAP
    .get(&compilation_id)
    .map_or_else(
      || "{}".to_string(),
      |state| {
        state
          .serialized
          .clone()
          .unwrap_or_else(|| serialize_import_meta_env_definitions(&state.definitions, None))
      },
    )
}

pub(crate) fn import_meta_env_definitions_string_with_properties(
  compilation_id: CompilationId,
  obj_keys: Option<&DestructuringAssignmentProperties>,
) -> String {
  let Some(obj_keys) = obj_keys else {
    return import_meta_env_definitions_string(compilation_id);
  };
  IMPORT_META_ENV_DEFINITIONS_MAP
    .get(&compilation_id)
    .map_or_else(
      || "{}".to_string(),
      |state| serialize_import_meta_env_definitions(&state.definitions, Some(obj_keys)),
    )
}

pub(crate) fn add_import_meta_env_value_dependency(parser: &mut JavascriptParser) {
  parser.build_info.value_dependencies.insert(
    IMPORT_META_ENV_VALUE_DEP_KEY.to_string(),
    import_meta_env_definitions_string(parser.compilation_id),
  );
}

pub(crate) fn import_meta_env_typeof_definition(
  parser: &mut JavascriptParser,
  name: &str,
) -> Option<String> {
  let state = IMPORT_META_ENV_DEFINITIONS_MAP.get(&parser.compilation_id)?;
  let code = state.definitions.typeof_definitions.get(name)?;
  parser.build_info.value_dependencies.insert(
    concat_string!(VALUE_DEP_PREFIX, TYPEOF_PREFIX, name),
    code.to_string(),
  );
  Some(code_to_string(code, None, None).into_owned())
}

pub(crate) fn is_import_meta_env_name(name: &str) -> bool {
  name == IMPORT_META_ENV || name.starts_with(IMPORT_META_ENV_PREFIX)
}

pub(crate) fn is_import_meta_env_member(member_expr: &MemberExpr) -> bool {
  member_expr
    .obj
    .as_meta_prop()
    .is_some_and(|meta_prop| meta_prop.kind == MetaPropKind::ImportMeta)
    && match &member_expr.prop {
      MemberProp::Ident(ident) => ident.sym == "env",
      MemberProp::Computed(computed) => computed
        .expr
        .as_lit()
        .and_then(|lit| lit.as_str())
        .is_some_and(|str_lit| str_lit.value.as_str() == Some("env")),
      _ => false,
    }
}

pub(crate) fn import_meta_env_key(name: &str) -> Option<&str> {
  name.strip_prefix(IMPORT_META_ENV_PREFIX)
}

pub(crate) fn render_import_meta_env_definitions(
  parser: &JavascriptParser,
  start: u32,
  obj_keys: Option<&DestructuringAssignmentProperties>,
) -> String {
  let definitions =
    import_meta_env_definitions_string_with_properties(parser.compilation_id, obj_keys);
  wrap_code(
    Cow::Owned(definitions),
    false,
    Some(!parser.is_asi_position(start)),
  )
  .into_owned()
}

pub(crate) fn render_import_meta_env_member_chain(
  parser: &JavascriptParser,
  members: &[Atom],
  members_optionals: &[bool],
  start: u32,
) -> Option<String> {
  debug_assert!(members.first().is_some_and(|member| member == "env"));
  let env_members = members.get(1..)?;
  if env_members.is_empty() {
    return Some(render_import_meta_env_definitions(parser, start, None));
  }

  if let Some(state) = IMPORT_META_ENV_DEFINITIONS_MAP.get(&parser.compilation_id) {
    for count in (1..=env_members.len()).rev() {
      if let Some(value) = state.definitions.get(&env_members[..count]) {
        if count == env_members.len() {
          return Some(
            code_to_string(value, Some(!parser.is_asi_position(start)), None).into_owned(),
          );
        }
        let value = match value {
          Value::Object(object) => {
            code_object_property_to_string(object, None, env_members[count].as_ref())
          }
          _ => code_to_string(value, None, None),
        };
        let remaining =
          property_access_with_optional(&env_members[count..], &members_optionals[count + 1..], 0);
        return Some(concat_string!("(", value, ")", remaining));
      }
    }
  }
  let property = env_members.first()?;
  let definitions = IMPORT_META_ENV_DEFINITIONS_MAP
    .get(&parser.compilation_id)
    .map_or_else(
      || "{ }".to_string(),
      |state| {
        code_object_property_to_string(&state.definitions.definitions, None, property.as_ref())
          .into_owned()
      },
    );
  let remaining = property_access_with_optional(
    env_members,
    members_optionals.get(1..).unwrap_or_default(),
    0,
  );
  Some(concat_string!("(", definitions, ")", remaining))
}

pub(crate) fn render_import_meta_env_expression_info(
  parser: &JavascriptParser,
  info: &ExpressionExpressionInfo,
  start: u32,
) -> Option<String> {
  if !matches!(
    &info.root_info,
    ExportedVariableInfo::Name(root) if root == "import.meta"
  ) || info.members.first().is_none_or(|member| member != "env")
  {
    return None;
  }
  render_import_meta_env_member_chain(parser, &info.members, &info.members_optionals, start)
}

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::ops::Sub;

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde_json::json;
use swc_core::ecma::atoms::Atom;

use crate::{
  get_import_var, property_access, to_comment, to_normal_comment, AsyncDependenciesBlockId,
  ChunkGraph, Compilation, DependenciesBlock, DependencyId, ExportsArgument, ExportsType,
  FakeNamespaceObjectMode, InitFragmentExt, InitFragmentKey, InitFragmentStage, ModuleGraph,
  ModuleIdentifier, NormalInitFragment, RuntimeCondition, RuntimeGlobals, RuntimeSpec,
  TemplateContext,
};

pub fn runtime_condition_expression(
  chunk_graph: &ChunkGraph,
  runtime_condition: Option<&RuntimeCondition>,
  runtime: Option<&RuntimeSpec>,
  runtime_requirements: &mut RuntimeGlobals,
) -> String {
  let Some(runtime_condition) = runtime_condition else {
    return "true".to_string();
  };

  if let RuntimeCondition::Boolean(v) = runtime_condition {
    return v.to_string();
  }

  let mut positive_runtime_ids = HashSet::default();
  for_each_runtime(
    runtime,
    |runtime| {
      if let Some(runtime_id) =
        runtime.and_then(|runtime| chunk_graph.get_runtime_id(runtime.clone()))
      {
        positive_runtime_ids.insert(runtime_id);
      }
    },
    false,
  );

  let mut negative_runtime_ids = HashSet::default();
  for_each_runtime(
    subtract_runtime(runtime, runtime_condition.as_spec()).as_ref(),
    |runtime| {
      if let Some(runtime_id) =
        runtime.and_then(|runtime| chunk_graph.get_runtime_id(runtime.clone()))
      {
        negative_runtime_ids.insert(runtime_id);
      }
    },
    false,
  );

  runtime_requirements.insert(RuntimeGlobals::RUNTIME_ID);

  compile_boolean_matcher_from_lists(
    positive_runtime_ids.into_iter().collect::<Vec<_>>(),
    negative_runtime_ids.into_iter().collect::<Vec<_>>(),
  )(RuntimeGlobals::RUNTIME_ID.to_string())
}

fn compile_boolean_matcher_from_lists(
  positive_items: Vec<String>,
  negative_items: Vec<String>,
) -> Box<dyn Fn(String) -> String> {
  if positive_items.is_empty() {
    Box::new(|_| "false".to_string())
  } else if negative_items.is_empty() {
    Box::new(|_| "true".to_string())
  } else if positive_items.len() == 1 {
    let item = to_simple_string(&positive_items[0]);
    Box::new(move |value| format!("{} == {}", item, value))
  } else if negative_items.len() == 1 {
    let item = to_simple_string(&negative_items[0]);
    Box::new(move |value| format!("{} != {}", item, value))
  } else {
    let positive_regexp = items_to_regexp(positive_items);
    let negative_regexp = items_to_regexp(negative_items);

    if positive_regexp.len() <= negative_regexp.len() {
      Box::new(move |value| format!("/^{}$/.test({})", positive_regexp, value))
    } else {
      Box::new(move |value| format!("!/^{}$/.test({})", negative_regexp, value))
    }
  }
}

fn to_simple_string(input: &str) -> String {
  if input
    .parse::<f64>()
    .map_or(false, |n| input == n.to_string())
  {
    input.to_string()
  } else {
    serde_json::to_string(input).unwrap_or_default()
  }
}

fn subtract_runtime(a: Option<&RuntimeSpec>, b: Option<&RuntimeSpec>) -> Option<RuntimeSpec> {
  match (a, b) {
    (Some(a), None) => Some(a.clone()),
    (None, None) => None,
    (None, Some(b)) => Some(b.clone()),
    (Some(a), Some(b)) => Some(a.sub(b)),
  }
}

pub fn for_each_runtime<F>(runtime: Option<&RuntimeSpec>, mut f: F, deterministic_order: bool)
where
  F: FnMut(Option<&String>),
{
  match runtime {
    None => f(None),
    Some(runtime) => {
      if deterministic_order {
        let mut runtimes = runtime.iter().collect::<Vec<_>>();
        runtimes.sort();
        for r in runtimes {
          f(Some(&r.to_string()));
        }
      } else {
        for r in runtime {
          f(Some(&r.to_string()));
        }
      }
    }
  }
}

/// AOT regex optimization, copy from webpack https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/util/compileBooleanMatcher.js#L134-L233
pub(crate) fn items_to_regexp(items_arr: Vec<String>) -> String {
  if items_arr.len() == 1 {
    return quote_meta(&items_arr[0]);
  }

  let mut finished_items = Vec::new();
  let mut items_set: Vec<&str> = items_arr.iter().map(|s| s.as_str()).collect();
  items_set.sort();

  // Merge single char items: (a|b|c|d|ef) => ([abcd]|ef)
  let count_of_single_char_items = items_set.iter().filter(|&item| item.len() == 1).count();

  // Special case for only single char items
  if count_of_single_char_items == items_set.len() {
    let mut items_arr = items_set.into_iter().collect::<Vec<_>>();
    items_arr.sort();
    let single_char_items = items_arr.join("");
    return format!("[{}]", quote_meta(&single_char_items));
  }

  // align with js insertion order https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/util/compileBooleanMatcher.js#L152
  let mut items = items_arr.iter().cloned().collect::<BTreeSet<_>>();

  if count_of_single_char_items > 2 {
    let mut single_char_items: String = String::new();
    let mut new_items = BTreeSet::new();
    for item in items {
      if item.len() == 1 {
        single_char_items += &item;
        continue;
      }
      new_items.insert(item);
    }
    items = new_items;
    finished_items.push(format!("[{}]", quote_meta(&single_char_items)));
  }

  // Special case for 2 items with common prefix/suffix
  if finished_items.is_empty() && items.len() == 2 {
    let prefix = get_common_prefix(items.iter().map(|item| item.as_str()));
    let suffix = get_common_suffix(items.iter().map(|item| &item[prefix.len()..]));

    if !prefix.is_empty() || !suffix.is_empty() {
      return format!(
        "{}{}{}",
        quote_meta(&prefix),
        items_to_regexp(
          items
            .iter()
            .map(|item| item
              .strip_prefix(&prefix)
              .expect("should strip prefix")
              .to_string())
            .collect::<Vec<_>>()
        ),
        quote_meta(&suffix)
      );
    }
  }

  // Special case for 2 items with common suffix https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/util/compileBooleanMatcher.js#L178-L189
  if finished_items.is_empty() && items.len() == 2 {
    let mut it = items.iter();
    let a = it.next().expect("should have two element");
    let b = it.next().expect("should have two element");

    if !a.is_empty()
      && !b.is_empty()
      && a.ends_with(
        b.chars()
          .last()
          .expect("should have last char since b is not empty"),
      )
    {
      return format!(
        "{}{}",
        items_to_regexp(vec![
          a[0..a.len() - 1].to_string(),
          b[0..b.len() - 1].to_string()
        ]),
        quote_meta(&a[a.len() - 1..])
      );
    }
  }

  // Find common prefix: (a1|a2|a3|a4|b5) => (a(1|2|3|4)|b5)
  let prefixed = pop_common_items(
    &mut items,
    |item| {
      if !item.is_empty() {
        Some(
          item
            .chars()
            .next()
            .expect("should have at least one char")
            .to_string(),
        )
      } else {
        None
      }
    },
    |list| {
      if list.len() >= 3 {
        true
      } else if list.len() <= 1 {
        false
      } else {
        list[0].chars().nth(1) == list[1].chars().nth(1)
      }
    },
  );

  for prefixed_items in prefixed {
    let prefix = get_common_prefix(prefixed_items.iter().map(|item| item.as_str()));
    finished_items.push(format!(
      "{}{}",
      quote_meta(&prefix),
      items_to_regexp(
        prefixed_items
          .iter()
          .map(|item| item
            .strip_prefix(&prefix)
            .expect("should strip prefix")
            .to_string())
          .collect::<Vec<_>>()
      )
    ));
  }

  // Find common suffix: (a1|b1|c1|d1|e2) => ((a|b|c|d)1|e2)
  let suffixed = pop_common_items(
    &mut items,
    |item| {
      if !item.is_empty() {
        Some(item[item.len() - 1..].to_string())
      } else {
        None
      }
    },
    |list| {
      if list.len() >= 3 {
        true
      } else if list.len() <= 1 {
        false
      } else {
        list[0].chars().skip(list[0].len() - 2).collect::<String>()
          == list[1].chars().skip(list[1].len() - 2).collect::<String>()
      }
    },
  );

  for suffixed_items in suffixed {
    let suffix = get_common_suffix(suffixed_items.iter().map(|item| item.as_str()));
    finished_items.push(format!(
      "{}{}",
      items_to_regexp(
        suffixed_items
          .iter()
          .map(|item| item
            .strip_suffix(&suffix)
            .expect("should strip suffix")
            .to_string())
          .collect::<Vec<_>>()
      ),
      quote_meta(&suffix)
    ));
  }

  // TODO(from webpack) further optimize regexp, i.e., use ranges: (1|2|3|4|a) => [1-4a]
  let conditional = finished_items
    .into_iter()
    .chain(items.iter().map(|item| quote_meta(item)))
    .collect::<Vec<String>>();

  if conditional.len() == 1 {
    conditional[0].clone()
  } else {
    format!("({})", conditional.join("|"))
  }
}

fn quote_meta(s: &str) -> String {
  regex::escape(s)
}

fn pop_common_items<T, F, G>(items_set: &mut BTreeSet<T>, get_key: F, condition: G) -> Vec<Vec<T>>
where
  T: Clone + PartialEq + Eq + std::hash::Hash + Ord,
  F: Fn(&T) -> Option<String>,
  G: Fn(&[T]) -> bool,
{
  let mut map: HashMap<String, Vec<T>> = HashMap::default();

  for item in items_set.iter() {
    if let Some(key) = get_key(item) {
      let list = map.entry(key).or_default();
      list.push(item.clone());
    }
  }

  let mut result = Vec::new();

  for list in map.values() {
    if condition(list) {
      for item in list {
        items_set.remove(item);
      }
      result.push(list.clone());
    }
  }

  result
}

fn get_common_prefix<'a>(mut items: impl Iterator<Item = &'a str> + Clone) -> String {
  if items.clone().count() == 0 {
    return String::new();
  }

  let mut prefix = items
    .next()
    .expect("should have at least one element")
    .to_string();
  for item in items {
    for (p, c) in item.chars().enumerate() {
      if let Some(prefix_char) = prefix.chars().nth(p) {
        if c != prefix_char {
          prefix = prefix[..p].to_string();
          break;
        }
      } else {
        break;
      }
    }
  }

  prefix
}

fn get_common_suffix<'a, I: Iterator<Item = &'a str> + Clone>(mut items: I) -> String {
  if items.clone().count() == 0 {
    return String::new();
  }

  let mut suffix = items
    .next()
    .expect("should have at least one element")
    .to_string();
  for item in items {
    let mut p = item.len();
    let mut s = suffix.len();

    while s > 0 {
      s -= 1;
      if let Some(suffix_char) = suffix.chars().nth(s) {
        if let Some(item_char) = item.chars().nth(p - 1) {
          if item_char != suffix_char {
            suffix = suffix[s + 1..].to_string();
            break;
          }
        } else {
          break;
        }
      } else {
        break;
      }

      p -= 1;
    }
  }

  suffix
}

#[allow(clippy::too_many_arguments)]
pub fn export_from_import(
  code_generatable_context: &mut TemplateContext,
  default_interop: bool,
  request: &str,
  import_var: &str,
  mut export_name: Vec<Atom>,
  id: &DependencyId,
  is_call: bool,
  call_context: bool,
) -> String {
  let TemplateContext {
    runtime_requirements,
    compilation,
    init_fragments,
    module,
    runtime,
    ..
  } = code_generatable_context;
  let Some(module_identifier) = compilation
    .module_graph
    .module_identifier_by_dependency_id(id)
    .copied()
  else {
    return missing_module(request);
  };
  let is_new_treeshaking = compilation.options.is_new_tree_shaking();

  let exports_type = get_exports_type(&compilation.module_graph, id, &module.identifier());

  if default_interop {
    if !export_name.is_empty()
      && let Some(first_export_name) = export_name.first()
      && first_export_name == "default"
    {
      match exports_type {
        ExportsType::Dynamic => {
          if is_call {
            return format!("{import_var}_default(){}", property_access(export_name, 1));
          } else {
            return format!(
              "({import_var}_default(){})",
              property_access(export_name, 1)
            );
          }
        }
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
          export_name = export_name[1..].to_vec();
        }
        _ => {}
      }
    } else if !export_name.is_empty() {
      if matches!(exports_type, ExportsType::DefaultOnly) {
        return format!(
          "/* non-default import from non-esm module */undefined\n{}",
          property_access(export_name, 1)
        );
      } else if !matches!(exports_type, ExportsType::Namespace)
        && let Some(first_export_name) = export_name.first()
        && first_export_name == "__esModule"
      {
        return "/* __esModule */true".to_string();
      }
    } else if matches!(
      exports_type,
      ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
    ) {
      runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
      init_fragments.push(
        NormalInitFragment::new(
          format!("var {import_var}_namespace_cache;\n",),
          InitFragmentStage::StageHarmonyExports,
          -1,
          InitFragmentKey::unique(),
          None,
        )
        .boxed(),
      );
      return format!("/*#__PURE__*/ ({import_var}_namespace_cache || ({import_var}_namespace_cache = {}({import_var}{})))", RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT, if matches!(exports_type, ExportsType::DefaultOnly) { "" } else { ", 2" });
    }
  }

  if !export_name.is_empty() {
    let used_name = if is_new_treeshaking {
      let exports_info_id = compilation
        .module_graph
        .get_exports_info(&module_identifier)
        .id;
      let used = exports_info_id.get_used_name(
        &compilation.module_graph,
        *runtime,
        crate::UsedName::Vec(export_name.clone()),
      );
      if let Some(used) = used {
        let used = match used {
          crate::UsedName::Str(str) => vec![str],
          crate::UsedName::Vec(strs) => strs,
        };
        Cow::Owned(used)
      } else {
        return format!(
          "{} undefined",
          to_normal_comment(&property_access(&export_name, 0))
        );
      }
    } else {
      Cow::Borrowed(&export_name)
    };
    let comment = if *used_name != export_name {
      to_normal_comment(&property_access(&export_name, 0))
    } else {
      "".to_string()
    };
    let property = property_access(&*used_name, 0);
    if is_call && !call_context {
      format!("(0, {import_var}{comment}{property})")
    } else {
      format!("{import_var}{comment}{property}")
    }
  } else {
    import_var.to_string()
  }
}

pub fn get_exports_type(
  module_graph: &ModuleGraph,
  id: &DependencyId,
  parent_module: &ModuleIdentifier,
) -> ExportsType {
  let module = module_graph
    .module_identifier_by_dependency_id(id)
    .expect("should have module");
  let strict = module_graph
    .module_by_identifier(parent_module)
    .expect("should have mgm")
    .get_strict_harmony_module();
  module_graph
    .module_by_identifier(module)
    .expect("should have mgm")
    .get_exports_type_readonly(module_graph, strict)
}

pub fn get_exports_type_with_strict(
  module_graph: &ModuleGraph,
  id: &DependencyId,
  strict: bool,
) -> ExportsType {
  let module = module_graph
    .module_identifier_by_dependency_id(id)
    .expect("should have module");
  module_graph
    .module_by_identifier(module)
    .expect("should have module")
    .get_exports_type_readonly(module_graph, strict)
}

pub fn module_id_expr(request: &str, module_id: &str) -> String {
  format!(
    "{}{}",
    to_comment(request),
    serde_json::to_string(module_id).expect("should render module id")
  )
}

pub fn module_id(
  compilation: &Compilation,
  id: &DependencyId,
  request: &str,
  weak: bool,
) -> String {
  if let Some(module_identifier) = compilation
    .module_graph
    .module_identifier_by_dependency_id(id)
    && let Some(module_id) = compilation.chunk_graph.get_module_id(*module_identifier)
  {
    module_id_expr(request, module_id)
  } else if weak {
    "null /* weak dependency, without id */".to_string()
  } else {
    missing_module(request)
  }
}

pub fn import_statement(
  code_generatable_context: &mut TemplateContext,
  id: &DependencyId,
  request: &str,
  update: bool, // whether a new variable should be created or the existing one updated
) -> (String, String) {
  let TemplateContext {
    runtime_requirements,
    compilation,
    module,
    ..
  } = code_generatable_context;
  if compilation
    .module_graph
    .module_identifier_by_dependency_id(id)
    .is_none()
  {
    return (missing_module_statement(request), "".to_string());
  };

  let module_id_expr = module_id(compilation, id, request, false);

  runtime_requirements.insert(RuntimeGlobals::REQUIRE);

  let import_var = get_import_var(&compilation.module_graph, *id);

  let opt_declaration = if update { "" } else { "var " };

  let import_content = format!(
    "/* harmony import */{opt_declaration}{import_var} = {}({module_id_expr});\n",
    RuntimeGlobals::REQUIRE
  );

  let exports_type = get_exports_type(&compilation.module_graph, id, &module.identifier());
  if matches!(exports_type, ExportsType::Dynamic) {
    runtime_requirements.insert(RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT);
    return (
      import_content,
      format!(
        "/* harmony import */{opt_declaration}{import_var}_default = /*#__PURE__*/{}({import_var});\n",
        RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT,
      ),
    );
  }
  (import_content, "".to_string())
}

pub fn module_namespace_promise(
  code_generatable_context: &mut TemplateContext,
  dep_id: &DependencyId,
  block: Option<&AsyncDependenciesBlockId>,
  request: &str,
  _message: &str,
  weak: bool,
) -> String {
  let TemplateContext {
    runtime_requirements,
    compilation,
    module,
    ..
  } = code_generatable_context;
  if compilation
    .module_graph
    .module_identifier_by_dependency_id(dep_id)
    .is_none()
  {
    return missing_module_promise(request);
  };

  let promise = block_promise(block, runtime_requirements, compilation);
  let exports_type = get_exports_type(&compilation.module_graph, dep_id, &module.identifier());
  let module_id_expr = module_id(compilation, dep_id, request, weak);

  let header = if weak {
    runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES);
    Some(format!(
      "if(!{}[{module_id_expr}]) {{\n {} \n}}",
      RuntimeGlobals::MODULE_FACTORIES,
      weak_error(request)
    ))
  } else {
    None
  };
  let mut fake_type = FakeNamespaceObjectMode::PROMISE_LIKE;
  let mut appending;
  match exports_type {
    ExportsType::Namespace => {
      if let Some(header) = header {
        appending = format!(
          ".then(function() {{ {header}\nreturn {}}})",
          module_raw(compilation, runtime_requirements, dep_id, request, weak)
        )
      } else {
        runtime_requirements.insert(RuntimeGlobals::REQUIRE);
        appending = format!(
          ".then({}.bind({}, {module_id_expr}))",
          RuntimeGlobals::REQUIRE,
          RuntimeGlobals::REQUIRE
        );
      }
    }
    _ => {
      if matches!(exports_type, ExportsType::Dynamic) {
        fake_type |= FakeNamespaceObjectMode::RETURN_VALUE;
      }
      if matches!(
        exports_type,
        ExportsType::DefaultWithNamed | ExportsType::Dynamic
      ) {
        fake_type |= FakeNamespaceObjectMode::MERGE_PROPERTIES;
      }
      runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
      if matches!(
        compilation.module_graph.is_async(
          compilation
            .module_graph
            .module_identifier_by_dependency_id(dep_id)
            .expect("should have module")
        ),
        Some(true)
      ) {
        if let Some(header) = header {
          appending = format!(
            ".then(function() {{\n {header}\nreturn {}\n}})",
            module_raw(compilation, runtime_requirements, dep_id, request, weak)
          )
        } else {
          runtime_requirements.insert(RuntimeGlobals::REQUIRE);
          appending = format!(
            ".then({}.bind({}, {module_id_expr}))",
            RuntimeGlobals::REQUIRE,
            RuntimeGlobals::REQUIRE
          );
        }
        appending.push_str(
          format!(
            ".then(function(m){{\n {}(m, {fake_type}) \n}})",
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT
          )
          .as_str(),
        );
      } else {
        fake_type |= FakeNamespaceObjectMode::MODULE_ID;
        if let Some(header) = header {
          let expr = format!(
            "{}({module_id_expr}, {fake_type}))",
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT
          );
          appending = format!(".then(function() {{\n {header} return {expr};\n}})");
        } else {
          runtime_requirements.insert(RuntimeGlobals::REQUIRE);
          appending = format!(
            ".then({}.bind({}, {module_id_expr}, {fake_type}))",
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
            RuntimeGlobals::REQUIRE
          );
        }
      }
    }
  }

  format!("{promise}{appending}")
}

pub fn block_promise(
  block: Option<&AsyncDependenciesBlockId>,
  runtime_requirements: &mut RuntimeGlobals,
  compilation: &Compilation,
) -> String {
  let Some(block) = block else {
    // ImportEagerDependency
    return "Promise.resolve()".to_string();
  };
  let chunk_group = compilation
    .chunk_graph
    .get_block_chunk_group(block, &compilation.chunk_group_by_ukey);
  let Some(chunk_group) = chunk_group else {
    return "Promise.resolve()".to_string();
  };
  if chunk_group.chunks.is_empty() {
    return "Promise.resolve()".to_string();
  }
  let chunks = chunk_group
    .chunks
    .iter()
    .map(|c| compilation.chunk_by_ukey.expect_get(c))
    .filter(|c| !c.has_runtime(&compilation.chunk_group_by_ukey) && c.id.is_some())
    .collect::<Vec<_>>();
  if chunks.len() == 1 {
    let chunk_id = serde_json::to_string(chunks[0].id.as_ref().expect("should have chunk.id"))
      .expect("should able to json stringify");
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    format!("{}({chunk_id})", RuntimeGlobals::ENSURE_CHUNK)
  } else if !chunks.is_empty() {
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    format!(
      "Promise.all([{}])",
      chunks
        .iter()
        .map(|c| format!(
          "{}({})",
          RuntimeGlobals::ENSURE_CHUNK,
          serde_json::to_string(c.id.as_ref().expect("should have chunk.id"))
            .expect("should able to json stringify")
        ))
        .collect::<Vec<_>>()
        .join(", ")
    )
  } else {
    "Promise.resolve()".to_string()
  }
}

pub fn module_raw(
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
  id: &DependencyId,
  request: &str,
  weak: bool,
) -> String {
  if let Some(module_identifier) = compilation
    .module_graph
    .module_identifier_by_dependency_id(id)
    && let Some(module_id) = compilation.chunk_graph.get_module_id(*module_identifier)
  {
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    format!(
      "{}({})",
      RuntimeGlobals::REQUIRE,
      module_id_expr(request, module_id)
    )
  } else if weak {
    weak_error(request)
  } else {
    missing_module(request)
  }
}

fn missing_module(request: &str) -> String {
  format!("Object({}())", throw_missing_module_error_function(request))
}

fn missing_module_statement(request: &str) -> String {
  format!("{};\n", missing_module(request))
}

fn missing_module_promise(request: &str) -> String {
  format!(
    "Promise.resolve().then({})",
    throw_missing_module_error_function(request)
  )
}

fn throw_missing_module_error_function(request: &str) -> String {
  format!(
    "function webpackMissingModule() {{ {} }}",
    throw_missing_module_error_block(request)
  )
}

pub fn throw_missing_module_error_block(request: &str) -> String {
  let e = format!("Cannot find module '{}'", request);
  format!(
    "var e = new Error({}); e.code = 'MODULE_NOT_FOUND'; throw e;",
    json!(e)
  )
}

fn weak_error(request: &str) -> String {
  format!("var e = new Error('Module is not available (weak dependency), request is {request}'); e.code = 'MODULE_NOT_FOUND'; throw e;")
}

pub fn returning_function(return_value: &str, args: &str) -> String {
  format!("function({args}) {{ return {return_value}; }}")
}

pub fn basic_function(args: &str, body: &str) -> String {
  format!("function({args}) {{\n{body}\n}}")
}

pub fn sync_module_factory(
  dep: &DependencyId,
  request: &str,
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
) -> String {
  let factory = returning_function(
    &module_raw(compilation, runtime_requirements, dep, request, false),
    "",
  );
  returning_function(&factory, "")
}

pub fn async_module_factory(
  block_id: &AsyncDependenciesBlockId,
  request: &str,
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
) -> String {
  let block = block_id.expect_get(compilation);
  let dep = block.get_dependencies()[0];
  let ensure_chunk = block_promise(Some(block_id), runtime_requirements, compilation);
  let factory = returning_function(
    &module_raw(compilation, runtime_requirements, &dep, request, false),
    "",
  );
  returning_function(
    &if ensure_chunk.starts_with("Promise.resolve(") {
      factory
    } else {
      format!("{ensure_chunk}.then({})", returning_function(&factory, ""))
    },
    "",
  )
}

pub fn define_es_module_flag_statement(
  exports_argument: ExportsArgument,
  runtime_requirements: &mut RuntimeGlobals,
) -> String {
  runtime_requirements.insert(RuntimeGlobals::MAKE_NAMESPACE_OBJECT);
  runtime_requirements.insert(RuntimeGlobals::EXPORTS);

  format!(
    "{}({});\n",
    RuntimeGlobals::MAKE_NAMESPACE_OBJECT,
    exports_argument
  )
}
#[allow(unused_imports)]
mod test_items_to_regexp {
  use super::items_to_regexp;
  #[test]
  fn basic() {
    assert_eq!(
      items_to_regexp(
        vec!["a", "b", "c", "d", "ef"]
          .into_iter()
          .map(String::from)
          .collect::<Vec<_>>(),
      ),
      "([abcd]|ef)".to_string()
    );

    assert_eq!(
      items_to_regexp(
        vec!["a1", "a2", "a3", "a4", "b5"]
          .into_iter()
          .map(String::from)
          .collect::<Vec<_>>(),
      ),
      "(a[1234]|b5)".to_string()
    );

    assert_eq!(
      items_to_regexp(
        vec!["a1", "b1", "c1", "d1", "e2"]
          .into_iter()
          .map(String::from)
          .collect::<Vec<_>>(),
      ),
      "([abcd]1|e2)".to_string()
    );

    assert_eq!(
      items_to_regexp(
        vec!["1", "2", "3", "4", "a"]
          .into_iter()
          .map(String::from)
          .collect::<Vec<_>>(),
      ),
      "[1234a]".to_string()
    );
  }
}

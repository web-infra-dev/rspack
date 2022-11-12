use hashbrown::{HashMap, HashSet};
use rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};

use crate::CodeGenerationResult;

pub const RUNTIME_PLACEHOLDER_INSTALLED_MODULES: &str = "{/* __INSTALLED_MODULES__*/}";
pub const RUNTIME_PLACEHOLDER_RSPACK_EXECUTE: &str = "/* RSPACK_EXECUTE */";

#[derive(Clone, Debug, Default)]
pub struct Runtime {
  pub sources: Vec<RawSource>,
  pub context_indent: String,
}

impl Runtime {
  pub fn generate(&self) -> BoxSource {
    let mut concat = self.sources.iter().fold(
      ConcatSource::new([RawSource::from("(function () { ")]),
      |mut concat, cur| {
        concat.add(cur.clone());
        concat
      },
    );
    concat.add(RawSource::from(" })();"));
    concat.boxed()
  }

  pub fn generate_rspack_execute(
    &self,
    namespace: &str,
    require_str: &str,
    ids: &[&String],
  ) -> BoxSource {
    let sources = ids
      .iter()
      .map(|id| {
        RawSource::from(format!(
          r#"{}["{}"].{}("{}");"#,
          self.context_indent, namespace, require_str, id
        ))
      })
      .collect::<Vec<_>>();
    let concat = ConcatSource::new(sources);
    concat.boxed()
  }

  pub fn web_generate_with_inline_modules(&mut self, modules_code: BoxSource) -> BoxSource {
    let runtime_source = self.generate().source().to_string();
    let execute_code_start = runtime_source
      .find(RUNTIME_PLACEHOLDER_RSPACK_EXECUTE)
      .unwrap();
    let execute_code_end = execute_code_start + RUNTIME_PLACEHOLDER_RSPACK_EXECUTE.len();

    ConcatSource::new([
      // runtime_source is all runtime code, and it's RawSource, so use RawSource at here is fine.
      RawSource::from(&runtime_source[0..execute_code_start]).boxed(),
      modules_code,
      RawSource::from(&runtime_source[execute_code_end..runtime_source.len()]).boxed(),
    ])
    .boxed()
  }

  pub fn node_generate_with_inline_modules(
    &mut self,
    modules_code: BoxSource,
    execute_code: BoxSource,
  ) -> BoxSource {
    let runtime_source = self.generate().source().to_string();
    let modules_code_start = runtime_source
      .find(RUNTIME_PLACEHOLDER_INSTALLED_MODULES)
      .unwrap();
    let modules_code_end = modules_code_start + RUNTIME_PLACEHOLDER_INSTALLED_MODULES.len();
    let execute_code_start = runtime_source
      .find(RUNTIME_PLACEHOLDER_RSPACK_EXECUTE)
      .unwrap();
    let execute_code_end = execute_code_start + RUNTIME_PLACEHOLDER_RSPACK_EXECUTE.len();
    ConcatSource::new([
      // runtime_source is all runtime code, and it's RawSource, so use RawSource at here is fine.
      RawSource::from(&runtime_source[0..modules_code_start]).boxed(),
      RawSource::from("{\n").boxed(),
      modules_code,
      RawSource::from("}").boxed(),
      RawSource::from(&runtime_source[modules_code_end..execute_code_start]).boxed(),
      execute_code,
      RawSource::from(&runtime_source[execute_code_end..runtime_source.len()]).boxed(),
    ])
    .boxed()
  }
}

pub type RuntimeSpec = HashSet<String>;
pub type RuntimeKey = String;

#[derive(Default, Debug)]
enum RuntimeMode {
  #[default]
  Empty = 0,
  SingleEntry = 1,
  Map = 2,
}

pub fn is_runtime_equal(a: &RuntimeSpec, b: &RuntimeSpec) -> bool {
  if a.len() != b.len() {
    return false;
  }

  let mut a: Vec<String> = Vec::from_iter(a.iter().cloned());
  let mut b: Vec<String> = Vec::from_iter(b.iter().cloned());

  a.sort();
  b.sort();

  a.into_iter().zip(b.into_iter()).all(|(a, b)| a == b)
}

pub fn get_runtime_key(runtime: RuntimeSpec) -> String {
  let mut runtime: Vec<String> = Vec::from_iter(runtime.into_iter());
  runtime.sort();
  runtime.join("\n")
}

#[derive(Default, Debug)]
pub struct RuntimeSpecMap {
  mode: RuntimeMode,
  map: HashMap<RuntimeKey, CodeGenerationResult>,

  single_runtime: Option<RuntimeSpec>,
  single_value: Option<CodeGenerationResult>,
}

impl RuntimeSpecMap {
  pub fn size(&self) -> usize {
    let mode = self.mode as usize;

    if mode <= 1 {
      mode
    } else {
      self.map.len()
    }
  }

  pub fn get(&self, runtime: &RuntimeSpec) -> Option<&CodeGenerationResult> {
    match self.mode {
      RuntimeMode::Empty => None,
      RuntimeMode::SingleEntry => {
        if let Some(single_runtime) = self.single_runtime.as_ref() && is_runtime_equal(single_runtime, runtime) {
          self.single_value.as_ref()
        } else {
          None
        }
      }
      RuntimeMode::Map => self.map.get(&get_runtime_key(runtime.clone())),
    }
  }

  pub fn set(&mut self, runtime: RuntimeSpec, value: CodeGenerationResult) {
    match self.mode {
      RuntimeMode::Empty => {
        self.mode = RuntimeMode::SingleEntry;
        self.single_runtime = Some(runtime);
        self.single_value = Some(value);
      }
      RuntimeMode::SingleEntry => {
        if let Some(single_runtime) = self.single_runtime.as_ref() && is_runtime_equal(single_runtime, &runtime) {
          self.single_value = Some(value);
        } else {
          self.mode = RuntimeMode::Map;

          let single_runtime = self.single_runtime.take().expect("Expected single runtime exists");
          let single_value = self.single_value.take().expect("Expected single value exists");

          self.map.insert(get_runtime_key(single_runtime), single_value);
        }
      }
      RuntimeMode::Map => {
        self.map.insert(get_runtime_key(runtime), value);
      }
    }
  }

  pub fn get_values(&self) -> Vec<&CodeGenerationResult> {
    match self.mode {
      RuntimeMode::Empty => vec![],
      RuntimeMode::SingleEntry => vec![self
        .single_value
        .as_ref()
        .expect("Expected single value exists")],
      RuntimeMode::Map => self.map.values().collect(),
    }
  }
}

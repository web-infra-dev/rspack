#![feature(let_chains)]
use std::borrow::Cow;

use json::{
  number::Number,
  object::Object,
  stringify,
  Error::{
    ExceededDepthLimit, FailedUtf8Parsing, UnexpectedCharacter, UnexpectedEndOfJson, WrongType,
  },
  JsonValue,
};
use rspack_core::{
  diagnostics::ModuleParseError,
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  BuildMetaDefaultObject, BuildMetaExportsType, CompilerOptions, ExportsInfo, GenerateContext,
  Module, ModuleGraph, ParserAndGenerator, Plugin, RuntimeGlobals, RuntimeSpec, SourceType,
  UsageState, NAMESPACE_OBJECT_EXPORT,
};
use rspack_error::{
  miette::diagnostic, DiagnosticExt, DiagnosticKind, IntoTWithDiagnosticArray, Result,
  TWithDiagnosticArray, TraceableError,
};

use crate::json_exports_dependency::JsonExportsDependency;

mod json_exports_dependency;
mod utils;

#[derive(Debug)]
struct JsonParserAndGenerator;

impl ParserAndGenerator for JsonParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn size(&self, module: &dyn Module, _source_type: &SourceType) -> f64 {
    module.original_source().map_or(0, |source| source.size()) as f64
  }

  fn parse(
    &mut self,
    parse_context: rspack_core::ParseContext,
  ) -> Result<TWithDiagnosticArray<rspack_core::ParseResult>> {
    let rspack_core::ParseContext {
      source: box_source,
      build_info,
      build_meta,
      loaders,
      ..
    } = parse_context;
    build_info.strict = true;
    build_meta.exports_type = BuildMetaExportsType::Default;
    // TODO default_object is not align with webpack
    build_meta.default_object = BuildMetaDefaultObject::RedirectWarn;
    let source = box_source.source();
    let strip_bom_source = source.strip_prefix('\u{feff}');
    let need_strip_bom = strip_bom_source.is_some();

    let parse_result = json::parse(strip_bom_source.unwrap_or(&source)).map_err(|e| {
      match e {
        UnexpectedCharacter { ch, line, column } => {
          let rope = ropey::Rope::from_str(&source);
          let line_offset = rope.try_line_to_byte(line - 1).expect("TODO:");
          let start_offset = source[line_offset..]
            .chars()
            .take(column)
            .fold(line_offset, |acc, cur| acc + cur.len_utf8());
          let start_offset = if need_strip_bom {
            start_offset + 1
          } else {
            start_offset
          };
          TraceableError::from_file(
            source.into_owned(),
            // one character offset
            start_offset,
            start_offset + 1,
            "Json parsing error".to_string(),
            format!("Unexpected character {ch}"),
          )
          .with_kind(DiagnosticKind::Json)
          .boxed()
        }
        ExceededDepthLimit | WrongType(_) | FailedUtf8Parsing => diagnostic!("{e}").boxed(),
        UnexpectedEndOfJson => {
          // End offset of json file
          let offset = source.len() - 1;
          TraceableError::from_file(
            source.into_owned(),
            offset,
            offset,
            "Json parsing error".to_string(),
            format!("{e}"),
          )
          .with_kind(DiagnosticKind::Json)
          .boxed()
        }
      }
    });

    let (diagnostics, data) = match parse_result {
      Ok(data) => (vec![], Some(data)),
      Err(err) => (
        vec![ModuleParseError::new(err, loaders).boxed().into()],
        None,
      ),
    };
    build_info.json_data = data.clone();

    Ok(
      rspack_core::ParseResult {
        presentational_dependencies: vec![],
        dependencies: if let Some(data) = data {
          vec![Box::new(JsonExportsDependency::new(data))]
        } else {
          vec![]
        },
        blocks: vec![],
        source: box_source,
        analyze_result: Default::default(),
        side_effects_bailout: None,
      }
      .with_diagnostic(diagnostics),
    )
  }

  // Safety: `ast_and_source` is available in code generation.
  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    _source: &BoxSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    let GenerateContext {
      compilation,
      runtime,
      concatenation_scope,
      ..
    } = generate_context;
    match generate_context.requested_source_type {
      SourceType::JavaScript => {
        generate_context
          .runtime_requirements
          .insert(RuntimeGlobals::MODULE);
        let module = compilation
          .get_module_graph()
          .module_by_identifier(&module.identifier())
          .expect("should have module identifier");
        let json_data = module
          .build_info()
          .as_ref()
          .and_then(|info| info.json_data.as_ref())
          .expect("should have json data");
        let exports_info = compilation
          .get_module_graph()
          .get_exports_info(&module.identifier());

        let final_json = match json_data {
          json::JsonValue::Object(_) | json::JsonValue::Array(_)
            if exports_info
              .other_exports_info
              .get_export_info(&compilation.get_module_graph())
              .get_used(*runtime)
              == UsageState::Unused =>
          {
            create_object_for_exports_info(
              json_data.clone(),
              exports_info,
              *runtime,
              &compilation.get_module_graph(),
            )
          }
          _ => json_data.clone(),
        };
        let is_js_object = final_json.is_object() || final_json.is_array();
        let final_json_string = stringify(final_json);
        let json_str = utils::escape_json(&final_json_string);
        let json_expr = if is_js_object && json_str.len() > 20 {
          Cow::Owned(format!(
            "JSON.parse('{}')",
            json_str.replace('\\', r"\\").replace('\'', r"\'")
          ))
        } else {
          json_str
        };
        let content = if let Some(ref mut scope) = concatenation_scope {
          scope.register_namespace_export(NAMESPACE_OBJECT_EXPORT);
          format!("var {NAMESPACE_OBJECT_EXPORT} = {json_expr}")
        } else {
          format!(r#"module.exports = {}"#, json_expr)
        };
        Ok(RawSource::from(content).boxed())
      }
      _ => panic!(
        "Unsupported source type: {:?}",
        generate_context.requested_source_type
      ),
    }
  }
}

#[derive(Debug)]
pub struct JsonPlugin;

impl Plugin for JsonPlugin {
  fn name(&self) -> &'static str {
    "json"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx.context.register_parser_and_generator_builder(
      rspack_core::ModuleType::Json,
      Box::new(|| Box::new(JsonParserAndGenerator {})),
    );

    Ok(())
  }
}

fn create_object_for_exports_info(
  data: JsonValue,
  exports_info: &ExportsInfo,
  runtime: Option<&RuntimeSpec>,
  mg: &ModuleGraph,
) -> JsonValue {
  if exports_info.other_exports_info.get_used(mg, runtime) != UsageState::Unused {
    return data;
  }

  match data {
    JsonValue::Null
    | JsonValue::Short(_)
    | JsonValue::String(_)
    | JsonValue::Number(_)
    | JsonValue::Boolean(_) => unreachable!(),
    JsonValue::Object(mut obj) => {
      let mut used_pair = vec![];
      for (key, value) in obj.iter_mut() {
        let export_info = exports_info.id.get_read_only_export_info(&key.into(), mg);
        let used = export_info.get_used(runtime);
        if used == UsageState::Unused {
          continue;
        }
        let new_value = if used == UsageState::OnlyPropertiesUsed
          && let Some(exports_info_id) = export_info.exports_info
        {
          let exports_info = mg.get_exports_info_by_id(&exports_info_id);
          // avoid clone
          let temp = std::mem::replace(value, JsonValue::Null);
          create_object_for_exports_info(temp, exports_info, runtime, mg)
        } else {
          std::mem::replace(value, JsonValue::Null)
        };
        let used_name = export_info
          .get_used_name(Some(&(key.into())), runtime)
          .expect("should have used name");
        used_pair.push((used_name, new_value));
      }
      let mut new_obj = Object::new();
      for (k, v) in used_pair {
        new_obj.insert(&k, v);
      }
      JsonValue::Object(new_obj)
    }
    JsonValue::Array(arr) => {
      let original_len = arr.len();
      let mut max_used_index = 0;
      let mut ret = arr
        .into_iter()
        .enumerate()
        .map(|(i, item)| {
          let export_info = exports_info
            .id
            .get_read_only_export_info(&format!("{i}").into(), mg);
          let used = export_info.get_used(runtime);
          if used == UsageState::Unused {
            return None;
          }
          max_used_index = max_used_index.max(i);
          if used == UsageState::OnlyPropertiesUsed
            && let Some(exports_info_id) = export_info.exports_info
          {
            let exports_info = mg.get_exports_info_by_id(&exports_info_id);
            Some(create_object_for_exports_info(
              item,
              exports_info,
              runtime,
              mg,
            ))
          } else {
            Some(item)
          }
        })
        .collect::<Vec<_>>();
      let arr_length_used = exports_info
        .id
        .get_read_only_export_info(&"length".into(), mg)
        .get_used(runtime);
      let array_length_when_used = match arr_length_used {
        UsageState::Unused => None,
        _ => Some(original_len),
      };
      let used_length = if let Some(array_length_when_used) = array_length_when_used {
        array_length_when_used
      } else {
        max_used_index + 1
      };
      ret.drain(used_length..);
      let normalized_ret = ret
        .into_iter()
        .map(|item| item.unwrap_or(JsonValue::Number(Number::from(0))))
        .collect::<Vec<_>>();
      JsonValue::Array(normalized_ret)
    }
  }
}

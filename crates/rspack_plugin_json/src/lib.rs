use std::borrow::Cow;

use cow_utils::CowUtils;
use json::{
  Error::{
    ExceededDepthLimit, FailedUtf8Parsing, UnexpectedCharacter, UnexpectedEndOfJson, WrongType,
  },
  JsonValue,
  number::Number,
  object::Object,
  stringify,
};
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, ChunkGraph, ExportsInfoGetter, GenerateContext,
  Module, ModuleGraph, NAMESPACE_OBJECT_EXPORT, ParseOption, ParserAndGenerator, Plugin,
  PrefetchExportsInfoMode, PrefetchedExportsInfoWrapper, RuntimeGlobals, RuntimeSpec, SourceType,
  UsageState, UsedNameItem,
  diagnostics::ModuleParseError,
  rspack_sources::{BoxSource, RawStringSource, Source, SourceExt},
};
use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray, error};
use rspack_util::itoa;

use crate::json_exports_dependency::JsonExportsDependency;

mod json_exports_dependency;
mod utils;

#[cacheable]
#[derive(Debug)]
struct JsonParserAndGenerator {
  pub exports_depth: u32,
  pub json_parse: bool,
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl ParserAndGenerator for JsonParserAndGenerator {
  fn source_types(&self, _module: &dyn Module, _module_graph: &ModuleGraph) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn size(&self, module: &dyn Module, _source_type: Option<&SourceType>) -> f64 {
    module
      .build_info()
      .json_data
      .as_ref()
      .map(|data| stringify(data.clone()).len() as f64)
      .unwrap_or(0.0)
  }

  async fn parse<'a>(
    &mut self,
    parse_context: rspack_core::ParseContext<'a>,
  ) -> Result<TWithDiagnosticArray<rspack_core::ParseResult>> {
    let rspack_core::ParseContext {
      source: box_source,
      build_info,
      build_meta,
      loaders,
      module_parser_options,
      ..
    } = parse_context;
    let source = box_source.source().into_string_lossy();
    let strip_bom_source = source.strip_prefix('\u{feff}');
    let need_strip_bom = strip_bom_source.is_some();
    let strip_bom_source = strip_bom_source.unwrap_or(&source);

    // If there is a custom parse, execute it to obtain the returned string.
    let parse_result_str = if let Some(p) = module_parser_options.and_then(|p| p.get_json()) {
      match &p.parse {
        ParseOption::Func(f) => {
          let parse_result = f(strip_bom_source.to_string()).await;
          parse_result.ok()
        }
        _ => None,
      }
    } else {
      None
    };

    let parse_result = json::parse(parse_result_str.as_deref().unwrap_or(strip_bom_source))
      .map_err(|e| {
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
            Error::from_string(
              Some(source.into_owned()),
              // one character offset
              start_offset,
              start_offset + 1,
              "JSON parse error".to_string(),
              format!("Unexpected character {ch}"),
            )
          }
          ExceededDepthLimit | WrongType(_) | FailedUtf8Parsing => error!("{}", e),
          UnexpectedEndOfJson => {
            // End offset of json file
            let length = source.len();
            let offset = if length > 0 { length - 1 } else { length };
            Error::from_string(
              Some(source.into_owned()),
              offset,
              offset,
              "JSON parse error".to_string(),
              format!("{e}"),
            )
          }
        }
      });

    let data = match parse_result {
      Ok(data) => data,
      Err(err) => {
        return Ok(
          rspack_core::ParseResult {
            presentational_dependencies: vec![],
            dependencies: vec![],
            blocks: vec![],
            code_generation_dependencies: vec![],
            source: box_source,
            side_effects_bailout: None,
          }
          .with_diagnostic(vec![
            Error::from(ModuleParseError::new(err, loaders)).into(),
          ]),
        );
      }
    };

    build_info.json_data = Some(data.clone());
    build_info.strict = true;
    build_meta.exports_type = BuildMetaExportsType::Default;
    build_meta.default_object = if data.is_object() || data.is_array() {
      // Ignore the json named exports warning, this violates standards, but other bundlers support it without warning.
      BuildMetaDefaultObject::RedirectWarn { ignore: true }
    } else {
      BuildMetaDefaultObject::False
    };

    Ok(
      rspack_core::ParseResult {
        presentational_dependencies: vec![],
        dependencies: vec![Box::new(JsonExportsDependency::new(
          data,
          self.exports_depth,
        ))],
        blocks: vec![],
        code_generation_dependencies: vec![],
        source: box_source,
        side_effects_bailout: None,
      }
      .with_diagnostic(vec![]),
    )
  }

  // Safety: `ast_and_source` is available in code generation.
  #[allow(clippy::unwrap_in_result)]
  async fn generate(
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
    let module_graph = compilation.get_module_graph();
    match generate_context.requested_source_type {
      SourceType::JavaScript => {
        let module = module_graph
          .module_by_identifier(&module.identifier())
          .expect("should have module identifier");
        let json_data = module
          .build_info()
          .json_data
          .as_ref()
          .expect("should have json data");
        let exports_info = module_graph
          .get_prefetched_exports_info(&module.identifier(), PrefetchExportsInfoMode::Default);

        let final_json = match json_data {
          json::JsonValue::Object(_) | json::JsonValue::Array(_)
            if matches!(
              exports_info.other_exports_info().get_used(*runtime),
              UsageState::Unused
            ) =>
          {
            create_object_for_exports_info(
              json_data.clone(),
              &exports_info,
              *runtime,
              &module_graph,
            )
          }
          _ => json_data.clone(),
        };
        let is_js_object = final_json.is_object() || final_json.is_array();
        let final_json_string = stringify(final_json);
        let json_str = utils::escape_json(&final_json_string);
        let json_expr = if self.json_parse && is_js_object && json_str.len() > 20 {
          Cow::Owned(format!(
            "JSON.parse('{}')",
            json_str.cow_replace('\\', r"\\").cow_replace('\'', r"\'")
          ))
        } else {
          json_str.cow_replace("\"__proto__\":", "[\"__proto__\"]:")
        };
        let content = if let Some(scope) = concatenation_scope {
          scope.register_namespace_export(NAMESPACE_OBJECT_EXPORT);
          format!("var {NAMESPACE_OBJECT_EXPORT} = {json_expr}")
        } else {
          generate_context
            .runtime_requirements
            .insert(RuntimeGlobals::MODULE);
          format!(r#"module.exports = {json_expr}"#)
        };
        Ok(RawStringSource::from(content).boxed())
      }
      _ => panic!(
        "Unsupported source type: {:?}",
        generate_context.requested_source_type
      ),
    }
  }

  fn get_concatenation_bailout_reason(
    &self,
    _module: &dyn Module,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    None
  }
}

#[derive(Debug)]
pub struct JsonPlugin;

impl Plugin for JsonPlugin {
  fn name(&self) -> &'static str {
    "json"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.register_parser_and_generator_builder(
      rspack_core::ModuleType::Json,
      Box::new(|p, g| {
        let p = p
          .and_then(|p| p.get_json())
          .expect("should have JsonParserOptions");

        let g = g
          .and_then(|g| g.get_json())
          .expect("should have JsonGeneratorOptions");

        Box::new(JsonParserAndGenerator {
          exports_depth: p.exports_depth.expect("should have exports_depth"),
          json_parse: g.json_parse.expect("should have json_parse"),
        })
      }),
    );

    Ok(())
  }
}

fn create_object_for_exports_info(
  data: JsonValue,
  exports_info: &PrefetchedExportsInfoWrapper<'_>,
  runtime: Option<&RuntimeSpec>,
  mg: &ModuleGraph,
) -> JsonValue {
  if exports_info.other_exports_info().get_used(runtime) != UsageState::Unused {
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
        let export_info = exports_info.get_read_only_export_info(&key.into());
        let used = export_info.get_used(runtime);
        if used == UsageState::Unused {
          continue;
        }
        let new_value = if used == UsageState::OnlyPropertiesUsed
          && let Some(exports_info) = export_info.exports_info()
        {
          // avoid clone
          let temp = std::mem::replace(value, JsonValue::Null);
          let exports_info =
            ExportsInfoGetter::prefetch(&exports_info, mg, PrefetchExportsInfoMode::Default);
          create_object_for_exports_info(temp, &exports_info, runtime, mg)
        } else {
          std::mem::replace(value, JsonValue::Null)
        };
        let UsedNameItem::Str(used_name) = export_info
          .get_used_name(Some(&(key.into())), runtime)
          .expect("should have used name")
        else {
          continue;
        };
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
          let mut i_buffer = itoa::Buffer::new();
          let i_str = i_buffer.format(i);
          let export_info = exports_info.get_read_only_export_info(&i_str.into());
          let used = export_info.get_used(runtime);
          if used == UsageState::Unused {
            return None;
          }
          max_used_index = max_used_index.max(i);
          if used == UsageState::OnlyPropertiesUsed
            && let Some(exports_info) = export_info.exports_info()
          {
            let exports_info =
              ExportsInfoGetter::prefetch(&exports_info, mg, PrefetchExportsInfoMode::Default);
            Some(create_object_for_exports_info(
              item,
              &exports_info,
              runtime,
              mg,
            ))
          } else {
            Some(item)
          }
        })
        .collect::<Vec<_>>();
      let arr_length_used = exports_info
        .get_read_only_export_info(&"length".into())
        .get_used(runtime);
      let array_length_when_used = match arr_length_used {
        UsageState::Unused => None,
        _ => Some(original_len),
      };
      let used_length = if let Some(array_length_when_used) = array_length_when_used {
        array_length_when_used
      } else {
        (max_used_index + 1).min(ret.len())
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

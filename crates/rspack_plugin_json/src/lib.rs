#![feature(let_chains)]
use json::{
  Error::{
    ExceededDepthLimit, FailedUtf8Parsing, UnexpectedCharacter, UnexpectedEndOfJson, WrongType,
  },
  JsonValue,
};
use rspack_core::{
  async_module_factory,
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  BuildMetaDefaultObject, BuildMetaExportsType, CompilerOptions, ExportsInfo, GenerateContext,
  Module, ModuleGraph, ParserAndGenerator, Plugin, RuntimeGlobals, RuntimeSpec, SourceType,
  UsageState,
};
use rspack_error::{
  internal_error, DiagnosticKind, Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray,
  TraceableError,
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
      resource_data,
      build_info,
      build_meta,
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
          Error::TraceableError(
            TraceableError::from_file(
              resource_data.resource_path.to_string_lossy().to_string(),
              source.into_owned(),
              // one character offset
              start_offset,
              start_offset + 1,
              "Json parsing error".to_string(),
              format!("Unexpected character {ch}"),
            )
            .with_kind(DiagnosticKind::Json),
          )
        }
        ExceededDepthLimit | WrongType(_) | FailedUtf8Parsing => {
          internal_error!(format!("{e}"))
        }
        UnexpectedEndOfJson => {
          // End offset of json file
          let offset = source.len();
          Error::TraceableError(
            TraceableError::from_file(
              resource_data.resource_path.to_string_lossy().to_string(),
              source.into_owned(),
              offset,
              offset,
              "Json parsing error".to_string(),
              format!("{e}"),
            )
            .with_kind(DiagnosticKind::Json),
          )
        }
      }
    });

    let (diagnostics, data) = match parse_result {
      Ok(data) => (vec![], Some(data)),
      Err(err) => (err.into(), None),
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
      }
      .with_diagnostic(diagnostics),
    )
  }

  // Safety: `ast_and_source` is available in code generation.
  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    source: &BoxSource,
    module: &dyn rspack_core::Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    let GenerateContext {
      compilation,
      module_generator_options,
      runtime_requirements,
      data,
      requested_source_type,
      runtime,
    } = generate_context;
    match generate_context.requested_source_type {
      SourceType::JavaScript => {
        generate_context
          .runtime_requirements
          .insert(RuntimeGlobals::MODULE);
        let mgm = compilation
          .module_graph
          .module_graph_module_by_identifier(&module.identifier())
          .expect("should have module identifier");
        let json_data = mgm
          .build_info
          .as_ref()
          .and_then(|info| info.json_data.as_ref())
          .expect("should have json data");
        let exports_info = compilation
          .module_graph
          .get_exports_info(&module.identifier());

        let final_json = match json_data {
          json::JsonValue::Object(_) | json::JsonValue::Array(_)
            if exports_info
              .other_exports_info
              // TODO: runtime opt
              .get_export_info(&compilation.module_graph)
              .get_used(*runtime)
              == UsageState::Unused =>
          {
            create_object_for_exports_info(
              json_data.clone(),
              exports_info,
              *runtime,
              &compilation.module_graph,
            )
          }
          _ => json_data.clone(),
        };

        let json_expr = final_json.to_string();
        Ok(
          RawSource::from(format!(
            r#"module.exports = {}"#,
            utils::escape_json(&json_expr)
          ))
          .boxed(),
        )
      }
      _ => Err(internal_error!(format!(
        "Unsupported source type {:?} for plugin Json",
        generate_context.requested_source_type,
      ))),
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
      let mut unused_key = vec![];
      for (key, value) in obj.iter_mut() {
        let export_info = exports_info.id.get_read_only_export_info(&key.into(), mg);
        let used = export_info.get_used(runtime);
        if used == UsageState::Unused {
          unused_key.push(key.to_string());
          continue;
        }
        if used == UsageState::OnlyPropertiesUsed
          && let Some(exports_info_id) = export_info.exports_info
        {
          let exports_info = mg.get_exports_info_by_id(&exports_info_id);
          // avoid clone
          let temp = std::mem::replace(value, JsonValue::Null);
          let ret = create_object_for_exports_info(temp, exports_info, runtime, mg);
          *value = ret;
        }
      }
      for k in unused_key {
        obj.remove(&k);
      }
      JsonValue::Object(obj)
    }
    JsonValue::Array(arr) => JsonValue::Array(
      arr
        .into_iter()
        .enumerate()
        .filter_map(|(i, item)| {
          let export_info = exports_info
            .id
            .get_read_only_export_info(&format!("{i}").into(), mg);
          let used = export_info.get_used(runtime);
          if used == UsageState::Unused {
            return None;
          }
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
        .collect::<Vec<_>>(),
    ),
  }
}

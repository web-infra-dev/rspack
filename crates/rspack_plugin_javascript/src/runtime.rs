use rayon::prelude::*;
use rspack_core::rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_core::{
  to_normal_comment, BoxModule, ChunkInitFragments, ChunkUkey, Compilation, RuntimeGlobals,
  SourceType,
};
use rspack_error::{error, Result};
use rspack_util::diff_mode::is_diff_mode;
use rustc_hash::FxHashSet as HashSet;

use crate::{JsPlugin, RenderSource};

pub fn render_chunk_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  ordered_modules: &Vec<&BoxModule>,
  all_strict: bool,
) -> Result<Option<(BoxSource, ChunkInitFragments)>> {
  let mut module_code_array = ordered_modules
    .par_iter()
    .filter_map(|module| {
      render_module(compilation, chunk_ukey, module, all_strict, true)
        .transpose()
        .map(|result| result.map(|(s, f, a)| (module.identifier(), s, f, a)))
    })
    .collect::<Result<Vec<_>>>()?;

  if module_code_array.is_empty() {
    return Ok(None);
  }

  module_code_array.sort_unstable_by_key(|(module_identifier, _, _, _)| *module_identifier);

  let chunk_init_fragments = module_code_array.iter().fold(
    ChunkInitFragments::default(),
    |mut chunk_init_fragments, (_, _, fragments, additional_fragments)| {
      chunk_init_fragments.extend((*fragments).clone());
      chunk_init_fragments.extend(additional_fragments.clone());
      chunk_init_fragments
    },
  );

  let module_sources: Vec<_> = module_code_array
    .into_iter()
    .map(|(_, source, _, _)| source)
    .collect();
  let module_sources = module_sources
    .into_par_iter()
    .fold(ConcatSource::default, |mut output, source| {
      output.add(source);
      output
    })
    .collect::<Vec<ConcatSource>>();

  let mut sources = ConcatSource::default();
  sources.add(RawSource::from("{\n"));
  sources.add(ConcatSource::new(module_sources));
  sources.add(RawSource::from("\n}"));

  Ok(Some((sources.boxed(), chunk_init_fragments)))
}

pub fn render_module(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &BoxModule,
  all_strict: bool,
  factory: bool,
) -> Result<Option<(BoxSource, ChunkInitFragments, ChunkInitFragments)>> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let code_gen_result = compilation
    .code_generation_results
    .get(&module.identifier(), Some(&chunk.runtime));
  let Some(origin_source) = code_gen_result.get(&SourceType::JavaScript) else {
    return Ok(None);
  };
  let hooks = JsPlugin::get_compilation_hooks(compilation);
  let mut module_chunk_init_fragments = ChunkInitFragments::default();
  let mut render_source = RenderSource {
    source: origin_source.clone(),
  };
  hooks.render_module_content.call(
    compilation,
    module,
    &mut render_source,
    &mut module_chunk_init_fragments,
  )?;
  let mut sources = ConcatSource::default();

  if factory {
    let runtime_requirements = compilation
      .chunk_graph
      .get_module_runtime_requirements(module.identifier(), &chunk.runtime);

    let need_module = runtime_requirements.is_some_and(|r| r.contains(RuntimeGlobals::MODULE));
    let need_exports = runtime_requirements.is_some_and(|r| r.contains(RuntimeGlobals::EXPORTS));
    let need_require = runtime_requirements.is_some_and(|r| {
      r.contains(RuntimeGlobals::REQUIRE) || r.contains(RuntimeGlobals::REQUIRE_SCOPE)
    });

    let mut args = Vec::new();
    if need_module || need_exports || need_require {
      let module_argument = module.get_module_argument();
      args.push(if need_module {
        module_argument.to_string()
      } else {
        format!("__unused_webpack_{module_argument}")
      });
    }

    if need_exports || need_require {
      let exports_argument = module.get_exports_argument();
      args.push(if need_exports {
        exports_argument.to_string()
      } else {
        format!("__unused_webpack_{exports_argument}")
      });
    }
    if need_require {
      args.push(RuntimeGlobals::REQUIRE.to_string());
    }
    let module_id = compilation
      .chunk_graph
      .get_module_id(module.identifier())
      .as_deref()
      .expect("should have module_id in render_module");
    sources.add(RawSource::from(
      serde_json::to_string(&module_id).map_err(|e| error!(e.to_string()))?,
    ));
    sources.add(RawSource::from(": "));
    if is_diff_mode() {
      sources.add(RawSource::from(format!(
        "\n{}\n",
        to_normal_comment(&format!("start::{}", module.identifier()))
      )));
    }
    sources.add(RawSource::from(format!(
      "(function ({}) {{\n",
      args.join(", ")
    )));
    if let Some(build_info) = &module.build_info()
      && build_info.strict
      && !all_strict
    {
      sources.add(RawSource::from("\"use strict\";\n"));
    }
    sources.add(render_source.source);
    sources.add(RawSource::from("\n\n})"));
    if is_diff_mode() {
      sources.add(RawSource::from(format!(
        "\n{}\n",
        to_normal_comment(&format!("end::{}", module.identifier()))
      )));
    }
    sources.add(RawSource::from(",\n"));
  } else {
    sources.add(render_source.source);
  }

  Ok(Some((
    sources.boxed(),
    code_gen_result.chunk_init_fragments.clone(),
    module_chunk_init_fragments,
  )))
}

pub fn render_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<BoxSource> {
  let runtime_modules_sources = render_runtime_modules(compilation, chunk_ukey)?;
  if runtime_modules_sources.source().is_empty() {
    return Ok(runtime_modules_sources);
  }

  let mut sources = ConcatSource::default();
  sources.add(RawSource::from(format!(
    "function({}) {{\n",
    RuntimeGlobals::REQUIRE
  )));
  sources.add(runtime_modules_sources);
  sources.add(RawSource::from("\n}\n"));
  Ok(sources.boxed())
}

pub fn render_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<BoxSource> {
  let mut sources = ConcatSource::default();
  compilation
    .chunk_graph
    .get_chunk_runtime_modules_in_order(chunk_ukey, compilation)
    .map(|(identifier, runtime_module)| {
      (
        compilation
          .runtime_module_code_generation_results
          .get(identifier)
          .expect("should have runtime module result"),
        runtime_module,
      )
    })
    .try_for_each(|((_, source), module)| -> Result<()> {
      if source.size() == 0 {
        return Ok(());
      }
      if is_diff_mode() {
        sources.add(RawSource::from(format!(
          "/* start::{} */\n",
          module.identifier()
        )));
      } else {
        sources.add(RawSource::from(format!("// {}\n", module.identifier())));
      }
      let supports_arrow_function = compilation
        .options
        .output
        .environment
        .supports_arrow_function();
      if module.should_isolate() {
        sources.add(RawSource::from(if supports_arrow_function {
          "(() => {\n"
        } else {
          "!function() {\n"
        }));
      }
      if module.cacheable() {
        sources.add(source.clone());
      } else {
        sources.add(module.generate_with_custom(compilation)?);
      }
      if module.should_isolate() {
        sources.add(RawSource::from(if supports_arrow_function {
          "\n})();\n"
        } else {
          "\n}();\n"
        }));
      }
      if is_diff_mode() {
        sources.add(RawSource::from(format!(
          "/* end::{} */\n",
          module.identifier()
        )));
      }
      Ok(())
    })?;
  Ok(sources.boxed())
}

pub fn stringify_chunks_to_array(chunks: &HashSet<String>) -> String {
  let mut v = Vec::from_iter(chunks.iter());
  v.sort_unstable();

  format!(
    r#"[{}]"#,
    v.iter().fold(String::new(), |prev, cur| {
      prev + format!(r#""{cur}","#).as_str()
    })
  )
}

pub fn stringify_array(vec: &[String]) -> String {
  format!(
    r#"[{}]"#,
    vec
      .iter()
      .map(|item| format!("\"{item}\""))
      .collect::<Vec<_>>()
      .join(", ")
  )
}

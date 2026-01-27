use std::{borrow::Cow, hash::Hash};

use rspack_core::{
  Chunk, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements, CompilationParams,
  CompilerCompilation, ExternalModule, ExternalRequest, Filename, LibraryAuxiliaryComment,
  LibraryCustomUmdObject, LibraryName, LibraryNonUmdObject, LibraryOptions, LibraryType,
  ModuleGraph, ModuleGraphCacheArtifact, PathData, Plugin, RuntimeGlobals, RuntimeModule,
  SourceType,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRender, JsPlugin, RenderSource,
};

use crate::utils::{external_arguments, externals_dep_array, get_options_for_chunk};

const PLUGIN_NAME: &str = "rspack.UmdLibraryPlugin";

#[derive(Debug)]
struct UmdLibraryPluginParsed<'a> {
  names: Cow<'a, LibraryCustomUmdObject>,
  auxiliary_comment: Option<&'a LibraryAuxiliaryComment>,
  named_define: Option<bool>,
}

#[plugin]
#[derive(Debug)]
pub struct UmdLibraryPlugin {
  optional_amd_external_as_global: bool,
  library_type: LibraryType,
}

impl UmdLibraryPlugin {
  pub fn new(optional_amd_external_as_global: bool, library_type: LibraryType) -> Self {
    Self::new_inner(optional_amd_external_as_global, library_type)
  }

  fn parse_options<'a>(&self, library: &'a LibraryOptions) -> UmdLibraryPluginParsed<'a> {
    let names = if let Some(LibraryName::UmdObject(names)) = &library.name {
      Cow::Borrowed(names)
    } else {
      let (single_name, root) = library
        .name
        .as_ref()
        .and_then(|n| match n {
          LibraryName::NonUmdObject(LibraryNonUmdObject::String(s)) => {
            Some((s.clone(), vec![s.clone()]))
          }
          LibraryName::NonUmdObject(LibraryNonUmdObject::Array(arr)) => {
            Some((arr.first()?.clone(), arr.clone()))
          }
          LibraryName::UmdObject(_) => unreachable!(),
        })
        .unzip();
      Cow::Owned(LibraryCustomUmdObject {
        commonjs: single_name.clone(),
        root,
        amd: single_name,
      })
    };
    UmdLibraryPluginParsed {
      names,
      auxiliary_comment: library.auxiliary_comment.as_ref(),
      named_define: library.umd_named_define,
    }
  }

  fn get_options_for_chunk<'a>(
    &self,
    compilation: &'a Compilation,
    chunk_ukey: &'a ChunkUkey,
  ) -> Option<UmdLibraryPluginParsed<'a>> {
    get_options_for_chunk(compilation, chunk_ukey)
      .filter(|library| library.library_type == self.library_type)
      .map(|library| self.parse_options(library))
  }
}

#[plugin_hook(CompilerCompilation for UmdLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.render.tap(render::new(self));
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRender for UmdLibraryPlugin)]
async fn render(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey) else {
    return Ok(());
  };
  let supports_arrow_function = compilation
    .options
    .output
    .environment
    .supports_arrow_function();
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let module_graph = compilation.get_module_graph();
  let module_graph_cache = &compilation.module_graph_cache_artifact;
  let modules = compilation
    .chunk_graph
    .get_chunk_modules_identifier(chunk_ukey)
    .iter()
    .filter_map(|identifier| {
      module_graph
        .module_by_identifier(identifier)
        .and_then(|module| module.as_external_module())
        .and_then(|m| {
          let ty = m.get_external_type();
          (ty == "umd" || ty == "umd2").then_some(m)
        })
    })
    .collect::<Vec<&ExternalModule>>();
  let mut optional_externals: Vec<&ExternalModule> = vec![];
  let mut required_externals = vec![];
  let externals = modules.clone();

  if self.optional_amd_external_as_global {
    for module in &externals {
      if module_graph.is_optional(&module.id, module_graph_cache) {
        optional_externals.push(*module);
      } else {
        required_externals.push(*module);
      }
    }
  } else {
    required_externals = externals.clone();
  }

  let amd_factory = if optional_externals.is_empty() {
    "factory".to_string()
  } else {
    let wrapper_arguments = external_arguments(&required_externals, compilation);
    let factory_arguments = if required_externals.is_empty() {
      externals_root_array(&optional_externals)?
    } else {
      format!(
        "{}, {}",
        external_arguments(&required_externals, compilation),
        externals_root_array(&optional_externals)?
      )
    };
    format!(
      r#"function webpackLoadOptionalExternalModuleAmd({wrapper_arguments}) {{
      return factory({factory_arguments});
    }}"#
    )
  };

  let UmdLibraryPluginParsed {
    names,
    auxiliary_comment,
    named_define,
  } = options;

  let define = if let (Some(amd), Some(_)) = &(&names.amd, named_define) {
    format!(
      "define({}, {}, {amd_factory});\n",
      library_name(&[amd.to_string()], chunk, compilation).await?,
      externals_dep_array(&required_externals)?
    )
  } else {
    format!(
      "define({}, {amd_factory});\n",
      externals_dep_array(&required_externals)?
    )
  };

  let name = if let Some(commonjs) = &names.commonjs {
    library_name(std::slice::from_ref(commonjs), chunk, compilation).await?
  } else if let Some(root) = &names.root {
    library_name(root, chunk, compilation).await?
  } else {
    "".to_string()
  };

  let factory = if names.commonjs.is_some() || names.root.is_some() {
    let commonjs_code = format!(
      "{}
      exports[{}] = factory({});\n",
      get_auxiliary_comment("commonjs", auxiliary_comment),
      name,
      externals_require_array("commonjs", &externals, module_graph, module_graph_cache)?,
    );
    let root_code = format!(
      "{}
      {} = factory({});",
      get_auxiliary_comment("root", auxiliary_comment),
      replace_keys(
        accessor_access(
          Some("root"),
          &names
            .root
            .clone()
            .or_else(|| names.commonjs.clone().map(|commonjs| vec![commonjs]))
            .unwrap_or_default(),
        ),
        chunk,
        compilation,
      )
      .await?,
      externals_root_array(&externals)?
    );
    format!(
      "}} else if(typeof exports === 'object'){{\n
          {commonjs_code}
      }} else {{\n
          {root_code}
      }}\n",
    )
  } else {
    let value = if externals.is_empty() {
      "var a = factory();\n".to_string()
    } else {
      format!(
        "var a = typeof exports === 'object' ? factory({}) : factory({});\n",
        externals_require_array("commonjs", &externals, module_graph, module_graph_cache)?,
        externals_root_array(&externals)?
      )
    };
    format!(
      "}} else {{
          {value}
          for(var i in a) (typeof exports === 'object' ? exports : root)[i] = a[i];\n
      }}\n"
    )
  };

  let mut source = ConcatSource::default();
  source.add(RawStringSource::from(
    "(function webpackUniversalModuleDefinition(root, factory) {\n",
  ));
  source.add(RawStringSource::from(format!(
    r#"{}
      if(typeof exports === 'object' && typeof module === 'object') {{
          module.exports = factory({});
      }}"#,
    get_auxiliary_comment("commonjs2", auxiliary_comment),
    externals_require_array("commonjs2", &externals, module_graph, module_graph_cache)?
  )));
  source.add(RawStringSource::from(format!(
    "else if(typeof define === 'function' && define.amd) {{
          {}
          {define}
          {factory}
      }})({}, {} {{
          return ",
    get_auxiliary_comment("amd", auxiliary_comment),
    compilation.options.output.global_object,
    if supports_arrow_function {
      format!("({}) =>", external_arguments(&externals, compilation))
    } else {
      format!("function({})", external_arguments(&externals, compilation))
    },
  )));
  source.add(render_source.source.clone());
  source.add(RawStringSource::from_static("\n})"));
  render_source.source = source.boxed();
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for UmdLibraryPlugin)]
async fn js_chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  let Some(_) = self.get_options_for_chunk(compilation, chunk_ukey) else {
    return Ok(());
  };
  PLUGIN_NAME.hash(hasher);
  compilation.options.output.library.hash(hasher);
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for UmdLibraryPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  _runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let Some(_) = self.get_options_for_chunk(compilation, chunk_ukey) else {
    return Ok(());
  };
  runtime_requirements.insert(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME);
  Ok(())
}

impl Plugin for UmdLibraryPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    Ok(())
  }
}

async fn library_name(v: &[String], chunk: &Chunk, compilation: &Compilation) -> Result<String> {
  let value =
    serde_json::to_string(v.last().expect("should have last")).expect("invalid module_id");
  replace_keys(value, chunk, compilation).await
}

async fn replace_keys(v: String, chunk: &Chunk, compilation: &Compilation) -> Result<String> {
  compilation
    .get_path(
      &Filename::from(v),
      PathData::default()
        .chunk_id_optional(chunk.id().map(|id| id.as_str()))
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_name_optional(chunk.name_for_filename_template())
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SourceType::JavaScript,
          compilation.options.output.hash_digest_length,
        )),
    )
    .await
}

fn externals_require_array(
  external_type: &str,
  externals: &[&ExternalModule],
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
) -> Result<String> {
  Ok(
    externals
      .iter()
      .map(|m| {
        let request = match &m.request {
          ExternalRequest::Single(r) => r,
          ExternalRequest::Map(map) => map
            .get(external_type)
            .ok_or_else(|| error!("Missing external configuration for type: {external_type}"))?,
        };
        let primary = serde_json::to_string(request.primary()).to_rspack_result()?;
        let mut expr = if let Some(rest) = request.rest() {
          format!("require({}){}", primary, &accessor_to_object_access(rest))
        } else {
          format!("require({primary})")
        };
        if module_graph.is_optional(&m.id, module_graph_cache) {
          expr = format!("(function webpackLoadOptionalExternalModule() {{ try {{ return {expr}; }} catch(e) {{}} }}())");
        }
        Ok(expr)
      })
      .collect::<Result<Vec<_>>>()?
      .join(", "),
  )
}

fn externals_root_array(modules: &[&ExternalModule]) -> Result<String> {
  Ok(
    modules
      .iter()
      .map(|m| {
        let external_type = "root";
        let request = match &m.request {
          ExternalRequest::Single(r) => r.iter(),
          ExternalRequest::Map(map) => map
            .get(external_type)
            .map(|r| r.iter())
            .ok_or_else(|| error!("Missing external configuration for type: {external_type}"))?,
        };
        Ok(format!("root{}", accessor_to_object_access(request)))
      })
      .collect::<Result<Vec<_>>>()?
      .join(", "),
  )
}

fn accessor_to_object_access<S: AsRef<str>>(accessor: impl IntoIterator<Item = S>) -> String {
  accessor
    .into_iter()
    .map(|s| {
      format!(
        "[{}]",
        serde_json::to_string(s.as_ref()).expect("failed to serde_json::to_string")
      )
    })
    .collect::<Vec<_>>()
    .join("")
}

fn accessor_access(base: Option<&str>, accessor: &[String]) -> String {
  accessor
    .iter()
    .enumerate()
    .map(|(i, _)| {
      let a = if let Some(base) = base {
        format!("{base}{}", accessor_to_object_access(&accessor[..(i + 1)]))
      } else {
        format!(
          "{}{}",
          accessor[0],
          accessor_to_object_access(&accessor[1..(i + 1)])
        )
      };
      if i == accessor.len() - 1 {
        return a;
      }
      if i == 0 && base.is_none() {
        return format!("{a} = typeof {a} === 'object' ? {a} : {{}}");
      }
      format!("{a} = {a} || {{}}")
    })
    .collect::<Vec<_>>()
    .join(", ")
}

fn get_auxiliary_comment(t: &str, auxiliary_comment: Option<&LibraryAuxiliaryComment>) -> String {
  if let Some(auxiliary_comment) = auxiliary_comment
    && let Some(value) = match t {
      "amd" => &auxiliary_comment.amd,
      "commonjs" => &auxiliary_comment.commonjs,
      "commonjs2" => &auxiliary_comment.commonjs2,
      "root" => &auxiliary_comment.root,
      _ => &None,
    }
  {
    return format!("\t// {value} \n");
  }
  "".to_string()
}

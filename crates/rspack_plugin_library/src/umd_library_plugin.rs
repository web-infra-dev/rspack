use std::hash::Hash;

use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  AdditionalChunkRuntimeRequirementsArgs, Chunk, Compilation, ExternalModule, Filename,
  JsChunkHashArgs, LibraryAuxiliaryComment, PathData, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  PluginRenderHookOutput, RenderArgs, RuntimeGlobals, SourceType,
};

use super::utils::{external_arguments, external_dep_array};

#[derive(Debug)]
pub struct UmdLibraryPlugin {
  _optional_amd_external_as_global: bool,
}

impl UmdLibraryPlugin {
  pub fn new(_optional_amd_external_as_global: bool) -> Self {
    Self {
      _optional_amd_external_as_global,
    }
  }
}

impl Plugin for UmdLibraryPlugin {
  fn name(&self) -> &'static str {
    "UmdLibraryPlugin"
  }

  fn additional_chunk_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    args
      .runtime_requirements
      .insert(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME);
    Ok(())
  }

  fn render(&self, _ctx: PluginContext, args: &RenderArgs) -> PluginRenderHookOutput {
    let compilation = &args.compilation;
    let chunk = args.chunk();
    let modules = compilation
      .chunk_graph
      .get_chunk_module_identifiers(args.chunk)
      .iter()
      .filter_map(|identifier| {
        compilation
          .module_graph
          .module_by_identifier(identifier)
          .and_then(|module| module.as_external_module())
      })
      .collect::<Vec<&ExternalModule>>();
    // TODO check if external module is optional
    let optional_externals: Vec<&ExternalModule> = vec![];
    let externals = modules.clone();
    let required_externals = modules.clone();

    let amd_factory = if optional_externals.is_empty() {
      "factory"
    } else {
      ""
    };

    let (name, umd_named_define, auxiliary_comment) =
      if let Some(library) = &compilation.options.output.library {
        (
          &library.name,
          &library.umd_named_define,
          &library.auxiliary_comment,
        )
      } else {
        (&None, &None, &None)
      };

    let (amd, commonjs, root) = if let Some(name) = &name {
      (&name.amd, &name.commonjs, &name.root)
    } else {
      (&None, &None, &None)
    };

    let define = if let (Some(amd), Some(_)) = &(amd, umd_named_define) {
      format!(
        "define({amd}, {}, {amd_factory});\n",
        external_dep_array(&required_externals)
      )
    } else {
      format!(
        "define({}, {amd_factory});\n",
        external_dep_array(&required_externals)
      )
    };

    let factory = if name.is_some() {
      let commonjs_code = format!(
        "{}
        exports[{}] = factory({});\n",
        get_auxiliary_comment("commonjs", auxiliary_comment),
        &commonjs
          .clone()
          .map(|commonjs| library_name(&[commonjs], chunk, compilation))
          .or_else(|| root
            .clone()
            .map(|root| library_name(&root, chunk, compilation)))
          .unwrap_or_default(),
        externals_require_array("commonjs", &externals),
      );
      let root_code = format!(
        "{}
        {} = factory({});",
        get_auxiliary_comment("root", auxiliary_comment),
        replace_keys(
          accessor_access(
            Some("root"),
            &root
              .clone()
              .or_else(|| commonjs.clone().map(|commonjs| vec![commonjs]))
              .unwrap_or_default(),
          ),
          chunk,
          compilation,
        ),
        external_root_array(&externals)
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
          externals_require_array("commonjs", &externals),
          external_root_array(&externals)
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
    source.add(RawSource::from(
      "(function webpackUniversalModuleDefinition(root, factory) {\n",
    ));
    source.add(RawSource::from(format!(
      r#"{}
        if(typeof exports === 'object' && typeof module === 'object') {{
            module.exports = factory({});
        }}"#,
      get_auxiliary_comment("commonjs2", auxiliary_comment),
      externals_require_array("commonjs2", &externals)
    )));
    source.add(RawSource::from(format!(
      "else if(typeof define === 'function' && define.amd) {{
            {}
            {define}
            {factory}
        }})({}, function({}) {{
            return ",
      get_auxiliary_comment("amd", auxiliary_comment),
      compilation.options.output.global_object,
      external_arguments(&externals, compilation)
    )));
    source.add(args.source.clone());
    source.add(RawSource::from("\n});"));
    Ok(Some(source.boxed()))
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    self.name().hash(&mut args.hasher);
    args
      .compilation
      .options
      .output
      .library
      .hash(&mut args.hasher);
    Ok(())
  }
}

fn library_name(v: &[String], chunk: &Chunk, compilation: &Compilation) -> String {
  let value = format!("'{}'", v.last().expect("should have last"));
  replace_keys(value, chunk, compilation)
}

fn replace_keys(v: String, chunk: &Chunk, compilation: &Compilation) -> String {
  compilation.get_path(
    &Filename::from(v),
    PathData::default().chunk(chunk).content_hash_optional(
      chunk
        .content_hash
        .get(&SourceType::JavaScript)
        .map(|i| i.as_str()),
    ),
  )
}

fn externals_require_array(_t: &str, externals: &[&ExternalModule]) -> String {
  externals
    .iter()
    .map(|m| {
      let request = &m.request;
      // TODO: check if external module is optional
      format!("require('{}')", request.as_str())
    })
    .collect::<Vec<_>>()
    .join(", ")
}

fn external_root_array(modules: &[&ExternalModule]) -> String {
  modules
    .iter()
    .map(|m| {
      let request = &m.request;
      format!(
        "root{}",
        accessor_to_object_access(&[request.as_str().to_owned()])
      )
    })
    .collect::<Vec<_>>()
    .join(", ")
}

fn accessor_to_object_access(accessor: &[String]) -> String {
  accessor
    .iter()
    .map(|s| format!("['{s}']"))
    .collect::<Vec<_>>()
    .join("")
}

fn accessor_access(base: Option<&str>, accessor: &Vec<String>) -> String {
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

fn get_auxiliary_comment(t: &str, auxiliary_comment: &Option<LibraryAuxiliaryComment>) -> String {
  if let Some(auxiliary_comment) = auxiliary_comment {
    if let Some(value) = match t {
      "amd" => &auxiliary_comment.amd,
      "commonjs" => &auxiliary_comment.commonjs,
      "commonjs2" => &auxiliary_comment.commonjs2,
      "root" => &auxiliary_comment.root,
      _ => &None,
    } {
      return format!("\t// {value} \n");
    }
  }
  "".to_string()
}

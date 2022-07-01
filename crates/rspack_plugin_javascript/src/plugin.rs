use crate::utils::parse_file;
use crate::visitors::ClearMark;
use crate::{module::JsModule, utils::get_swc_compiler};
use anyhow::{Ok, Result};
use rayon::prelude::*;
use rspack_core::{
  Asset, AssetFilename, ModuleType, NormalModuleFactoryContext, ParseModuleArgs, Parser, Plugin,
  PluginContext, PluginParseModuleHookOutput, PluginRenderManifestHookOutput, RspackAst,
};
use swc_common::comments::SingleThreadedComments;
use swc_common::Mark;
use swc_ecma_transforms::react::{react, Options as ReactOptions};
use swc_ecma_transforms::{react as swc_react, resolver};
use swc_ecma_visit::{as_folder, FoldWith};
use tracing::instrument;

#[derive(Debug)]
pub struct JsPlugin {}

impl Plugin for JsPlugin {
  fn apply(&mut self, ctx: PluginContext<&mut rspack_core::ApplyContext>) -> anyhow::Result<()> {
    ctx
      .context
      .register_parser(ModuleType::Js, Box::new(JsParser {}));
    ctx
      .context
      .register_parser(ModuleType::Ts, Box::new(JsParser {}));
    ctx
      .context
      .register_parser(ModuleType::Tsx, Box::new(JsParser {}));
    ctx
      .context
      .register_parser(ModuleType::Jsx, Box::new(JsParser {}));

    Ok(())
  }

  #[instrument(skip_all)]
  fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: rspack_core::RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let module_graph = &compilation.module_graph;
    let chunk = compilation
      .chunk_graph
      .chunk_by_id(args.chunk_id)
      .ok_or_else(|| anyhow::format_err!("Not found chunk {:?}", args.chunk_id))?;
    let ordered_modules = chunk.ordered_modules(module_graph);
    let code = ordered_modules
      .par_iter()
      .filter(|module| {
        matches!(
          module.module_type,
          ModuleType::Js | ModuleType::Ts | ModuleType::Tsx | ModuleType::Jsx | ModuleType::Css
        )
      })
      .map(|module| {
        if module.module_type.is_css() {
          // FIXME: Ugly workaround
          format!(
            r#"
          rs.define("{}", function(__rspack_require__, module, exports) {{
            "use strict";
        }});
          "#,
            module.id
          )
        } else {
          module.module.render(module, compilation)
        }
      })
      .chain([{
        if chunk.kind.is_entry() {
          format!(
            "rs.require(\"{}\")",
            ordered_modules
              .last()
              .ok_or_else(|| anyhow::format_err!("TODO:"))?
              .id
              .as_str()
          )
        } else {
          String::new()
        }
      }])
      .fold(String::new, |mut output, cur| {
        output += &cur;
        output
      })
      .collect();
    Ok(vec![Asset::new(
      code,
      AssetFilename::Static(format!("{}.js", args.chunk_id)),
    )])
  }
}

#[derive(Debug)]
struct JsParser {}

impl Parser for JsParser {
  fn parse(
    &self,
    module_type: ModuleType,
    args: ParseModuleArgs,
  ) -> anyhow::Result<rspack_core::BoxModule> {
    if !matches!(
      module_type,
      ModuleType::Js | ModuleType::Ts | ModuleType::Tsx | ModuleType::Jsx
    ) {
      anyhow::bail!(
        "`module_type` {:?} not supported for `JsParser`",
        module_type
      );
    }
    let ast = {
      match args.ast {
        Some(RspackAst::JavaScript(_ast)) => Ok(_ast),
        None => {
          if let Some(source) = args.source {
            Ok(parse_file(source, args.uri, &module_type))
          } else {
            anyhow::bail!("ast and source is both empty for {}", args.uri)
          }
        }
        _ => anyhow::bail!("not supported ast {:?} for js parser", args.ast),
      }
    }?;

    let ast = get_swc_compiler().run(|| {
      let top_level_mark = Mark::new();
      let mut react_folder = react::<SingleThreadedComments>(
        get_swc_compiler().cm.clone(),
        None,
        ReactOptions {
          development: Some(false),
          runtime: Some(swc_react::Runtime::Automatic),
          refresh: None,
          ..Default::default()
        },
        Mark::new(),
      );

      let ast = ast.fold_with(&mut resolver(Mark::new(), top_level_mark, false));
      let ast = ast.fold_with(&mut react_folder);
      ast.fold_with(&mut as_folder(ClearMark))
    });
    Ok(Box::new(JsModule {
      ast,
      uri: args.uri.to_string(),
      module_type,
    }))
  }
}

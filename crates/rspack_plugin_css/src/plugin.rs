// mod js_module;
// pub use js_module::*;

use crate::{
  module::{CssModule, CSS_MODULE_SOURCE_TYPE_LIST},
  SWC_COMPILER,
};

// use anyhow::{Context, Result};
use preset_env_base::query::{Query, Targets};
use rayon::prelude::*;
use rspack_core::{
  AssetContent, BoxModule, FilenameRenderOptions, ModuleRenderResult, ModuleType, ParseModuleArgs,
  Parser, Plugin, RenderManifestEntry, SourceType,
};
use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};

use swc_css::visit::VisitMutWith;

use swc_css_prefixer::{options::Options, prefixer};
#[derive(Debug, Default)]
pub struct CssPlugin {
  config: CssConfig,
}

#[derive(Debug, Default, Clone)]
pub struct CssConfig {
  pub preset_env: Vec<String>,
}

impl CssPlugin {
  pub fn new(config: CssConfig) -> Self {
    Self { config }
  }
}
impl Plugin for CssPlugin {
  fn name(&self) -> &'static str {
    "css"
  }

  fn apply(
    &mut self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    ctx.context.register_parser(
      ModuleType::Css,
      Box::new(CssParser {
        config: self.config.clone(),
      }),
    );
    Ok(())
  }

  // fn reuse_ast(&self) -> bool {
  //   true
  // }

  // fn transform_include(&self, uri: &str) -> bool {
  //   let extension = Path::new(uri).extension().unwrap().to_string_lossy();
  //   extension == "css"
  // }

  // fn transform(
  //   &self,
  //   _ctx: rspack_core::PluginContext<&mut NormalModuleFactoryContext>,
  //   args: rspack_core::TransformArgs,
  // ) -> rspack_core::PluginTransformOutput {
  //   if let Some(TransformAst::Css(mut ast)) = args.ast {
  //     if let Some(query) = self.get_query() {
  //       ast.visit_mut_with(&mut prefixer(Options {
  //         env: Some(Targets::Query(query)),
  //       }));
  //     }
  //     Ok({
  //       TransformResult {
  //         content: None,
  //         ast: Some(TransformAst::Css(ast)),
  //       }
  //     })
  //   } else {
  //     Ok({
  //       TransformResult {
  //         content: None,
  //         ast: args.ast,
  //       }
  //     })
  //   }
  // }
  // fn transform(
  //   &self,
  //   _ctx: rspack_core::PluginContext<&mut NormalModuleFactoryContext>,
  //   args: rspack_core::TransformArgs,
  // ) -> rspack_core::PluginTransformOutput {
  //   if let Some(TransformAst::Css(mut ast)) = args.ast {
  //     ast.visit_mut_with(&mut prefixer());
  //     Ok({
  //       TransformResult {
  //         content: None,
  //         ast: Some(TransformAst::Css(ast)),
  //       }
  //     })
  //   } else {
  //     Ok({
  //       TransformResult {
  //         content: None,
  //         ast: args.ast,
  //       }
  //     })
  //   }
  // }

  // fn parse(&self, uri: &str, content: &Content) -> rspack_core::PluginParseOutput {
  //   let content = content
  //     .to_owned()
  //     .try_into_string()
  //     .context("Unable to serialize content as string which is required by plugin css")?;
  //   let stylesheet = SWC_COMPILER.parse_file(uri, content)?;
  //   Ok(TransformAst::Css(stylesheet))
  // }

  fn render_manifest(
    &self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::RenderManifestArgs,
  ) -> rspack_core::PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let module_graph = &compilation.module_graph;
    let chunk = args.chunk();
    let ordered_modules = chunk.ordered_modules(module_graph);
    let code = ordered_modules
      .par_iter()
      .filter(|module| {
        module
          .module
          .source_types(module, compilation)
          .contains(&SourceType::Css)
      })
      .map(|module| module.module.render(SourceType::Css, module, compilation))
      .collect::<Result<Vec<_>>>()?
      .into_par_iter()
      .fold(String::new, |mut output, cur| {
        if let Some(ModuleRenderResult::Css(source)) = cur {
          output += "\n\n";
          output += &source;
        }
        output
      })
      .collect::<String>();

    if code.is_empty() {
      Ok(Default::default())
    } else {
      Ok(vec![RenderManifestEntry::new(
        AssetContent::String(code),
        compilation
          .options
          .output
          .filename
          .render(FilenameRenderOptions {
            filename: Some(args.chunk().id.to_owned()),
            extension: Some(".css".to_owned()),
            id: None,
          }),
      )])
    }
  }
}

#[derive(Debug)]
struct CssParser {
  config: CssConfig,
}

impl CssParser {
  pub fn get_query(&self) -> Option<Query> {
    // TODO: figure out if the prefixer visitMut is stateless
    // I need to clone the preset_env every time, due to I don't know if it is stateless
    // If it is true, I reduce this clone
    if !self.config.preset_env.is_empty() {
      Some(Query::Multiple(self.config.preset_env.clone()))
    } else {
      None
    }
  }
}

impl Parser for CssParser {
  fn parse(
    &self,
    _module_type: ModuleType,
    args: ParseModuleArgs,
  ) -> Result<TWithDiagnosticArray<BoxModule>> {
    let content = args.source.try_into_string().map_err(|_| {
      Error::InternalError(
        "Unable to serialize content as string which is required by plugin css".into(),
      )
    })?;
    let TWithDiagnosticArray {
      inner: mut stylesheet,
      diagnostic,
    } = SWC_COMPILER.parse_file(args.uri, content)?;

    if let Some(query) = self.get_query() {
      stylesheet.visit_mut_with(&mut prefixer(Options {
        env: Some(Targets::Query(query)),
      }));
    }

    let module: BoxModule = Box::new(CssModule {
      ast: stylesheet,
      source_type_list: CSS_MODULE_SOURCE_TYPE_LIST,
      meta: args
        .meta
        .and_then(|data| if data.is_empty() { None } else { Some(data) }),
    });

    Ok(module.with_diagnostic(diagnostic))
  }
}

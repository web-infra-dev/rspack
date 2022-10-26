use rspack_error::{Error, IntoTWithDiagnosticArray};
use rspack_sources::{RawSource, SourceExt};

use crate::{
  ApplyContext, ExternalType, FactorizeAndBuildArgs, ModuleType, NormalModule,
  NormalModuleFactoryContext, ParserAndGenerator, Plugin, PluginContext,
  PluginFactorizeAndBuildHookOutput, SourceType, Target, TargetPlatform,
};

#[derive(Debug)]
pub struct ExternalPlugin {}

#[derive(Debug)]
struct ExternalParserAndGenerator {
  specifier: String,
  external_type: ExternalType,
  target: Target,
}

impl ParserAndGenerator for ExternalParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    &[SourceType::JavaScript]
  }

  fn size(&self, _module: &NormalModule, _source_type: &SourceType) -> f64 {
    // copied from webpack `ExternalModule`
    // roughly for url
    42.0
  }

  fn parse(
    &mut self,
    _parse_context: crate::ParseContext,
  ) -> rspack_error::Result<rspack_error::TWithDiagnosticArray<crate::ParseResult>> {
    Ok(
      crate::ParseResult {
        dependencies: vec![],
        ast_or_source: RawSource::from(match self.external_type {
          ExternalType::NodeCommonjs => {
            format!(r#"module.exports = require("{}")"#, self.specifier)
          }
          ExternalType::Window => {
            format!("module.exports = window.{}", self.specifier)
          }
          ExternalType::Auto => match self.target.platform {
            TargetPlatform::BrowsersList
            | TargetPlatform::Web
            | TargetPlatform::WebWorker
            | TargetPlatform::None => format!("module.exports = {}", self.specifier),
            TargetPlatform::Node(_) => {
              format!(
                r#"module.exports = __rspack_require__.nr("{}")"#,
                self.specifier
              )
            }
          },
        })
        .boxed()
        .into(),
      }
      .with_empty_diagnostic(),
    )
  }

  fn generate(
    &self,
    _requested_source_type: crate::SourceType,
    ast_or_source: &crate::AstOrSource,
    _module: &crate::ModuleGraphModule,
    _compilation: &crate::Compilation,
  ) -> rspack_error::Result<crate::GenerationResult> {
    Ok(crate::GenerationResult {
      // Safety: We know this value comes from parser, so it is safe here.
      ast_or_source: ast_or_source.to_owned().try_into_source()?.into(),
    })
  }
}

#[async_trait::async_trait]
impl Plugin for ExternalPlugin {
  fn name(&self) -> &'static str {
    "external"
  }

  fn apply(&mut self, _ctx: PluginContext<&mut ApplyContext>) -> Result<(), Error> {
    Ok(())
  }

  // Todo The factorize_and_build hook is a temporary solution and will be replaced with the real factorize hook later
  // stage 1: we need move building function(parse,loader runner) out of normal module factory
  // stage 2: Create a new hook that is the same as factory in webpack and change factorize_and_build to that
  async fn factorize_and_build(
    &self,
    _ctx: PluginContext,
    args: FactorizeAndBuildArgs<'_>,
    job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginFactorizeAndBuildHookOutput {
    let target = &job_ctx.options.target;
    let external_type = &job_ctx.options.external_type;
    for external_item in &job_ctx.options.external {
      match external_item {
        crate::External::Object(eh) => {
          let specifier = args.dependency.detail.specifier.as_str();
          if let Some(value) = eh.get(specifier) {
            job_ctx.module_type = Some(ModuleType::Js);

            // FIXME: using normal module here is a dirty hack.
            let mut normal_module = NormalModule::new(
              specifier.to_owned(),
              specifier.to_owned(),
              specifier.to_owned(),
              ModuleType::Js,
              Box::new(ExternalParserAndGenerator {
                specifier: value.to_owned(),
                external_type: external_type.clone(),
                target: target.clone(),
              }),
              crate::ResourceData {
                resource: value.to_owned(),
                resource_path: value.to_owned(),
                resource_query: None,
                resource_fragment: None,
                ignored: false,
              },
              job_ctx.options.clone(),
            );

            // This hacks normal module to skip build, since external module is not exist on the filesystem.
            normal_module.set_skip_build(true);

            tracing::trace!("parsed module {:?}", normal_module);
            return Ok(Some((specifier.to_string(), normal_module)));
          }
        }
        _ => {
          return Ok(None);
        }
      }
    }
    Ok(None)
  }
}

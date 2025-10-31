use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct ReactServerComponentPlugin;

#[plugin_hook(CompilerFinishMake for ReactServerComponentPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  // react-server layer，如果在 ssr 环境下，则需要生成 ssr module
  // 如果是在 rsc 环境下，只需要生成 client module

  // 从入口处寻找 use client 模块
  if module.get_layer() == Some("react-server") {
    if let Some(rsc) = module.build_info().rsc {
      if rsc.rsc_type == RscType::Client {
        // 生成 client module，挂载到 client entry 上

        // 如果 module 所在环境为 ssr，则生成 ssr module，挂载到 ssr entry 上
      }
    }
  }

  Ok(())
}

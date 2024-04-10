use rspack_error::Result;
use rspack_hook::{define_hook, plugin, plugin_hook};

mod simple {
  use super::*;

  define_hook!(Render: AsyncSeriesBail(compilation: &Compilation, source: &mut Source) -> bool);

  struct Compilation {
    id: u32,
    render_hook: RenderHook,
  }

  struct Source {
    content: String,
  }

  #[plugin]
  #[derive(Default)]
  struct MyRenderPlugin;

  #[plugin_hook(Render for MyRenderPlugin)]
  async fn render(&self, compilation: &Compilation, source: &mut Source) -> Result<Option<bool>> {
    source.content += "plugin.render";
    source.content += &compilation.id.to_string();
    Ok(Some(true))
  }

  #[tokio::test]
  async fn test() -> Result<()> {
    let mut compilation = Compilation {
      id: 0,
      render_hook: RenderHook::default(),
    };
    let mut source = Source {
      content: String::new(),
    };
    let plugin = MyRenderPlugin::default();
    compilation.render_hook.tap(render::new(&plugin));
    let result = compilation
      .render_hook
      .call(&compilation, &mut source)
      .await?;
    assert_eq!(result, Some(true));
    assert_eq!(source.content, "plugin.render0");
    Ok(())
  }
}

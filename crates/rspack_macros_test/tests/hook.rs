use rspack_error::Result;
use rspack_hook::{define_hook, plugin, plugin_hook};

mod simple {
  use super::*;

  define_hook!(Render: SeriesBail(compilation: &Compilation, source: &mut Source) -> bool);

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

mod sync_series {
  use super::*;

  define_hook!(Render: Sync(compilation: &Compilation, source: &mut Source));

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
  fn render(&self, compilation: &Compilation, source: &mut Source) -> Result<()> {
    source.content += "plugin.render";
    source.content += &compilation.id.to_string();
    Ok(())
  }

  #[test]
  fn test() -> Result<()> {
    let mut compilation = Compilation {
      id: 1,
      render_hook: RenderHook::default(),
    };
    let mut source = Source {
      content: String::new(),
    };
    let plugin = MyRenderPlugin::default();
    compilation.render_hook.tap(render::new(&plugin));
    compilation.render_hook.call(&compilation, &mut source)?;
    assert_eq!(source.content, "plugin.render1");
    Ok(())
  }
}

mod stage_order {
  use rspack_hook::Hook as _;

  use super::*;

  define_hook!(Render: Sync(source: &mut String));

  struct Tap {
    label: &'static str,
    stage: i32,
  }

  impl Render for Tap {
    fn run(&self, source: &mut String) -> Result<()> {
      source.push_str(self.label);
      Ok(())
    }

    fn stage(&self) -> i32 {
      self.stage
    }
  }

  struct AdditionalTaps;

  impl rspack_hook::Interceptor<RenderHook> for AdditionalTaps {
    fn call_blocking(
      &self,
      _hook: &RenderHook,
    ) -> Result<Vec<<RenderHook as rspack_hook::Hook>::Tap>> {
      Ok(vec![
        Box::new(Tap {
          label: "D",
          stage: 5,
        }),
        Box::new(Tap {
          label: "E",
          stage: 10,
        }),
      ])
    }
  }

  #[test]
  fn sorts_base_taps_at_registration() -> Result<()> {
    let mut hook = RenderHook::default();
    hook.tap(Tap {
      label: "A",
      stage: 10,
    });
    hook.tap(Tap {
      label: "B",
      stage: 0,
    });
    hook.tap(Tap {
      label: "C",
      stage: 10,
    });

    let mut source = String::new();
    hook.call(&mut source)?;
    assert_eq!(source, "BAC");
    Ok(())
  }

  #[test]
  fn sorts_additional_taps_by_stage_indices() -> Result<()> {
    let mut hook = RenderHook::default();
    hook.tap(Tap {
      label: "A",
      stage: 10,
    });
    hook.tap(Tap {
      label: "B",
      stage: 0,
    });
    hook.tap(Tap {
      label: "C",
      stage: 10,
    });
    hook.intercept(AdditionalTaps);

    let mut source = String::new();
    hook.call(&mut source)?;
    assert_eq!(source, "BDACE");
    Ok(())
  }
}

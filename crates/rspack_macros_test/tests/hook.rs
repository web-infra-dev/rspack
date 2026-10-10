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

mod waterfall_with_result {
  use rspack_hook::Hook as _;

  use super::*;

  #[derive(Default)]
  struct Data {
    visits: Vec<&'static str>,
  }

  define_hook!(Run: SeriesWaterfallWithResult(data: Box<Data>) -> Box<Data>, tracing=false);

  struct Tap {
    label: &'static str,
    stage: i32,
    fail: bool,
  }

  #[async_trait::async_trait]
  impl Run for Tap {
    async fn run(&self, mut data: Box<Data>) -> (Box<Data>, Result<()>) {
      data.visits.push(self.label);
      let result = if self.fail {
        Err(rspack_error::error!("tap failed"))
      } else {
        Ok(())
      };
      (data, result)
    }

    fn stage(&self) -> i32 {
      self.stage
    }
  }

  struct AdditionalTaps {
    fail_interceptor: bool,
    fail_tap: bool,
  }

  #[async_trait::async_trait]
  impl rspack_hook::Interceptor<RunHook> for AdditionalTaps {
    async fn call(&self, _hook: &RunHook) -> Result<Vec<<RunHook as rspack_hook::Hook>::Tap>> {
      if self.fail_interceptor {
        return Err(rspack_error::error!("interceptor failed"));
      }
      Ok(vec![Box::new(Tap {
        label: "additional",
        stage: 5,
        fail: self.fail_tap,
      })])
    }
  }

  #[tokio::test]
  async fn returns_ownership_without_taps() -> Result<()> {
    let data = Box::new(Data {
      visits: vec!["initial"],
    });
    let address = &*data as *const _;
    let (data, result) = RunHook::default().call(data).await;
    result?;
    assert_eq!(&*data as *const _, address);
    assert_eq!(data.visits, ["initial"]);
    Ok(())
  }

  #[tokio::test]
  async fn retains_data_and_stops_after_a_failed_tap() {
    for intercepted in [false, true] {
      let mut hook = RunHook::default();
      hook.tap(Tap {
        label: "first",
        stage: 0,
        fail: !intercepted,
      });
      hook.tap(Tap {
        label: "last",
        stage: 10,
        fail: false,
      });
      if intercepted {
        hook.intercept(AdditionalTaps {
          fail_interceptor: false,
          fail_tap: true,
        });
      }
      let data = Box::<Data>::default();
      let address = &*data as *const _;
      let (data, result) = hook.call(data).await;
      assert!(result.is_err());
      assert_eq!(&*data as *const _, address);
      let expected = if intercepted {
        vec!["first", "additional"]
      } else {
        vec!["first"]
      };
      assert_eq!(data.visits, expected);
    }
  }

  #[tokio::test]
  async fn runs_owned_taps_in_stage_order() -> Result<()> {
    let mut hook = RunHook::default();
    for (label, stage) in [("last", 10), ("first", 0)] {
      hook.tap(Tap {
        label,
        stage,
        fail: false,
      });
    }
    hook.intercept(AdditionalTaps {
      fail_interceptor: false,
      fail_tap: false,
    });
    let (data, result) = hook.call(Box::default()).await;
    result?;
    assert_eq!(data.visits, ["first", "additional", "last"]);
    Ok(())
  }

  #[tokio::test]
  async fn retains_data_when_an_interceptor_fails() {
    let mut hook = RunHook::default();
    hook.tap(Tap {
      label: "must not run",
      stage: 0,
      fail: false,
    });
    hook.intercept(AdditionalTaps {
      fail_interceptor: true,
      fail_tap: false,
    });
    let data = Box::new(Data {
      visits: vec!["initial"],
    });
    let address = &*data as *const _;
    let (data, result) = hook.call(data).await;
    assert!(result.is_err());
    assert_eq!(&*data as *const _, address);
    assert_eq!(data.visits, ["initial"]);
  }
}

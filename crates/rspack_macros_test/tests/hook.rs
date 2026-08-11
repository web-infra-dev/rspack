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

mod js_tap_register {
  use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  };

  use rspack_hook::{Hook as _, JsTapRegister};

  use super::*;

  define_hook!(Render: Series(source: &mut String));
  define_hook!(Other: Series());

  struct Tap {
    label: &'static str,
    stage: i32,
  }

  #[async_trait::async_trait]
  impl Render for Tap {
    async fn run(&self, source: &mut String) -> Result<()> {
      source.push_str(self.label);
      Ok(())
    }

    fn stage(&self) -> i32 {
      self.stage
    }
  }

  struct OtherTap;

  #[async_trait::async_trait]
  impl Other for OtherTap {
    async fn run(&self) -> Result<()> {
      Ok(())
    }
  }

  struct AdditionalTaps;

  #[async_trait::async_trait]
  impl rspack_hook::Interceptor<RenderHook> for AdditionalTaps {
    async fn call(
      &self,
      _hook: &RenderHook,
    ) -> Result<Vec<<RenderHook as rspack_hook::Hook>::Tap>> {
      Ok(vec![Box::new(Tap {
        label: "interceptor",
        stage: 5,
      })])
    }
  }

  #[derive(Default)]
  struct RegisterState {
    tap_count: AtomicUsize,
    call_count: AtomicUsize,
  }

  impl RegisterState {
    fn tap_count(&self) -> usize {
      self.tap_count.load(Ordering::Acquire)
    }
  }

  #[tokio::test]
  async fn skips_empty_js_register_without_calling_it() -> Result<()> {
    let state = Arc::new(RegisterState::default());
    let mut hook = RenderHook::default();
    hook.load_js_tap_register(JsTapRegister::new_async::<
      RegisterState,
      <RenderHook as rspack_hook::Hook>::Tap,
      _,
      _,
    >(
      state.clone(),
      RegisterState::tap_count,
      |state, _stages| async move {
        state.call_count.fetch_add(1, Ordering::Relaxed);
        Ok(vec![Box::new(Tap {
          label: "js",
          stage: 0,
        }) as <RenderHook as rspack_hook::Hook>::Tap])
      },
    ))?;

    assert!(hook.is_empty());
    assert!(!hook.has_js_taps());

    let mut source = String::new();
    hook.call(&mut source).await?;
    assert!(source.is_empty());
    assert_eq!(state.call_count.load(Ordering::Relaxed), 0);

    state.tap_count.store(1, Ordering::Release);
    assert!(!hook.is_empty());
    assert!(hook.has_js_taps());

    hook.call(&mut source).await?;
    assert_eq!(source, "js");
    assert_eq!(state.call_count.load(Ordering::Relaxed), 1);
    Ok(())
  }

  #[tokio::test]
  async fn merges_rust_interceptor_and_js_taps_by_stage() -> Result<()> {
    let state = Arc::new(RegisterState {
      tap_count: AtomicUsize::new(1),
      call_count: AtomicUsize::new(0),
    });
    let mut hook = RenderHook::default();
    hook.tap(Tap {
      label: "rust",
      stage: 5,
    });
    hook.intercept(AdditionalTaps);
    hook.load_js_tap_register(JsTapRegister::new_async::<
      RegisterState,
      <RenderHook as rspack_hook::Hook>::Tap,
      _,
      _,
    >(
      state.clone(),
      RegisterState::tap_count,
      |state, _stages| async move {
        state.call_count.fetch_add(1, Ordering::Relaxed);
        Ok(vec![Box::new(Tap {
          label: "js",
          stage: 5,
        }) as <RenderHook as rspack_hook::Hook>::Tap])
      },
    ))?;

    let mut source = String::new();
    hook.call(&mut source).await?;
    assert_eq!(source, "rustinterceptorjs");
    assert_eq!(state.call_count.load(Ordering::Relaxed), 1);
    Ok(())
  }

  #[tokio::test]
  async fn runs_rust_taps_without_loading_an_empty_js_register() -> Result<()> {
    let state = Arc::new(RegisterState::default());
    let mut hook = RenderHook::default();
    hook.tap(Tap {
      label: "rust",
      stage: 0,
    });
    hook.load_js_tap_register(JsTapRegister::new_async::<
      RegisterState,
      <RenderHook as rspack_hook::Hook>::Tap,
      _,
      _,
    >(
      state.clone(),
      RegisterState::tap_count,
      |state, _stages| async move {
        state.call_count.fetch_add(1, Ordering::Relaxed);
        Ok(vec![Box::new(Tap {
          label: "js",
          stage: 0,
        }) as <RenderHook as rspack_hook::Hook>::Tap])
      },
    ))?;

    let mut source = String::new();
    hook.call(&mut source).await?;
    assert_eq!(source, "rust");
    assert_eq!(state.call_count.load(Ordering::Relaxed), 0);
    Ok(())
  }

  #[test]
  fn rejects_a_register_for_another_hook_tap_type() {
    let state = Arc::new(RegisterState::default());
    let register =
      JsTapRegister::new_async::<RegisterState, <OtherHook as rspack_hook::Hook>::Tap, _, _>(
        state,
        RegisterState::tap_count,
        |_state, _stages| async move {
          Ok(vec![
            Box::new(OtherTap) as <OtherHook as rspack_hook::Hook>::Tap
          ])
        },
      );

    let error = RenderHook::default()
      .load_js_tap_register(register)
      .expect_err("tap types should be validated when loading a register");
    assert!(error.to_string().contains("type does not match"));
  }
}

mod blocking_js_tap_register {
  use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  };

  use rspack_hook::{Hook as _, JsTapRegister};

  use super::*;

  define_hook!(Render: Sync(source: &mut String));

  struct Tap;

  impl Render for Tap {
    fn run(&self, source: &mut String) -> Result<()> {
      source.push_str("js");
      Ok(())
    }
  }

  struct RegisterState(AtomicUsize);

  impl RegisterState {
    fn tap_count(&self) -> usize {
      self.0.load(Ordering::Acquire)
    }
  }

  #[test]
  fn supports_sync_hooks_without_calling_async_code() -> Result<()> {
    let state = Arc::new(RegisterState(AtomicUsize::new(1)));
    let mut hook = RenderHook::default();
    hook.load_js_tap_register(JsTapRegister::new_blocking::<
      RegisterState,
      <RenderHook as rspack_hook::Hook>::Tap,
      _,
    >(state, RegisterState::tap_count, |_state, _stages| {
      Ok(vec![Box::new(Tap) as <RenderHook as rspack_hook::Hook>::Tap])
    }))?;

    let mut source = String::new();
    hook.call(&mut source)?;
    assert_eq!(source, "js");
    Ok(())
  }
}

mod empty_hook_returns {
  use super::*;

  define_hook!(EmptySeries: Series());
  define_hook!(EmptySync: Sync());
  define_hook!(EmptyBail: SeriesBail() -> bool);
  define_hook!(EmptyWaterfall: SeriesWaterfall(data: String) -> String);

  #[tokio::test]
  async fn preserves_each_hook_kinds_empty_return() -> Result<()> {
    EmptySeriesHook::default().call().await?;
    EmptySyncHook::default().call()?;
    assert_eq!(EmptyBailHook::default().call().await?, None);
    assert_eq!(
      EmptyWaterfallHook::default()
        .call("input".to_string())
        .await?,
      "input"
    );
    Ok(())
  }
}

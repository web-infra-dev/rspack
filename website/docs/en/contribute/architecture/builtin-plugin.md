# Builtin Plugin

Builtin plugin uses [rspack_macros](https://github.com/web-infra-dev/rspack/tree/7cc39cc4bb6f73791a5bcb175137ffd84b105da5/crates/rspack_macros) to help you avoid writing boilerplate code, you can use [cargo-expand](https://github.com/dtolnay/cargo-expand) or [rust-analyzer expand macro](https://rust-analyzer.github.io/manual.html#expand-macro-recursively) to checkout the expanded code, and for developing/testing these macro, you can starts with [rspack_macros_test](https://github.com/web-infra-dev/rspack/tree/7cc39cc4bb6f73791a5bcb175137ffd84b105da5/crates/rspack_macros_test).

A simple example:

```rust
use rspack_hook::{plugin, plugin_hook};
use rspack_core::{Plugin, PluginContext, ApplyContext, CompilerOptions};
use rspack_core::CompilerCompilation;
use rspack_error::Result;

// define the plugin
#[plugin]
pub struct MyPlugin {
  options: MyPluginOptions
}

// define the plugin hook
#[plugin_hook(CompilerCompilation for MuPlugin)]
async fn compilation(&self, compilation: &mut Compilation) -> Result<()> {
  // do something...
}

// implement apply method for the plugin
impl Plugin for MyPlugin {
  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &mut CompilerOptions) -> Result<()> {
    ctx.context.compiler_hooks.tap(compilation::new(self))
    Ok(())
  }
}
```

And here is [an example](https://github.com/web-infra-dev/rspack/blob/7cc39cc4bb6f73791a5bcb175137ffd84b105da5/crates/rspack_plugin_ignore/src/lib.rs).

If the hook you need is not defined yet, you can define it by `rspack_hook::define_hook`, `compiler.hooks.assetEmitted` for example:

```rust
// this will allow you define hook's arguments without limit
define_hook!(CompilerShouldEmit: AsyncSeriesBail(compilation: &mut Compilation) -> bool);
//           ------------------  --------------- -----------------------------  -------
//           hook name           exec kind       hook arguments                 return value (Result<Option<bool>>)

#[derive(Debug, Default)]
pub struct CompilerHooks {
  // ...
  // and add it here
  pub asset_emitted: CompilerAssetEmittedHook,
}
```

There are 5 kinds of exec kind:

- AsyncSeries, return value is `Result<()>`
- AsyncSeriesBail, return value is `Result<Option<T>>`
- AsyncParallel, return value is `Result<()>`
- SyncSeries, return value is `Result<()>`
- SyncSeriesBail, return value is `Result<Option<T>>`

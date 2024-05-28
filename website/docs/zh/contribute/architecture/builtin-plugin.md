# Builtin 插件

Builtin 插件使用 [rspack_macros](https://github.com/web-infra-dev/rspack/tree/7cc39cc4bb6f73791a5bcb175137ffd84b105da5/crates/rspack_macros) 来帮助你避免写重复的代码, 你可以使用 [cargo-expand](https://github.com/dtolnay/cargo-expand) 或者 [rust-analyzer expand macro](https://rust-analyzer.github.io/manual.html#expand-macro-recursively) 来检查展开后的代码，并且开发/测试这些宏， 你可以使用 [rspack_macros_test](https://github.com/web-infra-dev/rspack/tree/7cc39cc4bb6f73791a5bcb175137ffd84b105da5/crates/rspack_macros_test) 来开始.

一个小例子如下:

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

并且这里也有 [一个例子](https://github.com/web-infra-dev/rspack/blob/7cc39cc4bb6f73791a5bcb175137ffd84b105da5/crates/rspack_plugin_ignore/src/lib.rs).

如果你需要的钩子还没有定义，你可以通过 `rspack_hook::define_hook`, `compiler.hooks.assetEmitted` 来定义它，例如：

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

执行的类型有 5 种：

- AsyncSeries，返回值为 `Result<()>`
- AsyncSeriesBail，返回值为 `Result<Option<T>>`
- AsyncParallel，返回值为 `Result<()>`
- SyncSeries，返回值为 `Result<()>`
- SyncSeriesBail，返回值为 `Result<Option<T>>`

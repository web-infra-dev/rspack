# rspack_binding_builder_macros

Exports a few macros to help you create a custom plugin register with Rspack.

## Important

This crate is used for creating a custom binding. It is not intended to be used directly by non-custom binding users.

## Guide

[Rspack Custom binding](https://rspack-contrib.github.io/rspack-rust-book/custom-binding/getting-started/index.html)

## Usage

### register_plugin

Create a custom plugin register with Rspack.

The created plugin register will be exposed to the final N-API binding.

The plugin needs to be wrapped with `require('@rspack/core').experiments.createNativePlugin`
to be used in the host.

#### Parameters

`register_plugin` macro accepts two arguments:

- The name of the plugin
- A resolver function that returns a `rspack_core::BoxPlugin`

The resolver function accepts two arguments:

- `env`: The environment of the plugin, it is the same as `napi::bindgen_prelude::Env`
- `options`: The options of the plugin, it is the same as `napi::bindgen_prelude::Unknown<'_>`

The resolver function should return a `rspack_core::BoxPlugin`

#### Example

This example will expose `registerMyBannerPlugin` in the final N-API binding:

> **Note:** The following example requires the `napi_derive`, `rspack_binding_builder_macros`, `rspack_core`, and `rspack_error` crates, and is for illustration only. It may not compile without additional setup.

```rust,ignore
use napi_derive::napi;
use rspack_binding_builder_macros::register_plugin;

register_plugin!(
  "MyBannerPlugin",
  |env: napi::bindgen_prelude::Env, options: napi::bindgen_prelude::Unknown<'_>| {
    Ok(Box::new(MyBannerPlugin) as rspack_core::BoxPlugin)
  }
);

#[derive(Debug)]
struct MyBannerPlugin;

impl rspack_core::Plugin for MyBannerPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> rspack_error::Result<()> {
    Ok(())
  }
}
```

The `registerMyBannerPlugin` function will be exposed to the final N-API binding.

```js
const { registerMyBannerPlugin } = require('your-custom-binding');

const plugin = registerMyBannerPlugin();
```

To actually use the plugin, you need to wrap it with `require('@rspack/core').experiments.createNativePlugin`:

```js
require('@rspack/core').experiments.createNativePlugin(
  'MyBannerPlugin',
  (options) => options,
);
```

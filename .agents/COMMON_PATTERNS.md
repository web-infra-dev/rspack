# Common Patterns

Common code patterns and templates used in Rspack.

## Plugin Implementation

### Basic Plugin Structure

```rust
use rspack_core::{Compilation, Plugin, ApplyContext};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[derive(Debug)]
pub struct MyPluginOptions {
 pub option1: String,
 pub option2: Option<bool>,
}

#[plugin]
#[derive(Debug)]
pub struct MyPlugin {
 config: MyPluginOptions,
}

impl MyPlugin {
 pub fn new(config: MyPluginOptions) -> Self {
  Self::new_inner(config)
 }
}

#[plugin_hook(CompilationProcessAssets for MyPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
 // Hook implementation
 Ok(())
}

impl Plugin for MyPlugin {
 fn name(&self) -> &'static str {
  "rspack.MyPlugin"
 }

 fn apply(&self, ctx: &mut ApplyContext<'_>) -> Result<()> {
  ctx.compilation_hooks.process_assets.tap(process_assets::new(self));
  Ok(())
 }
}
```

### Plugin with Multiple Hooks

```rust
#[plugin_hook(CompilationProcessAssets for MyPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
 Ok(())
}

#[plugin_hook(CompilationEmit for MyPlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
 Ok(())
}

impl Plugin for MyPlugin {
 fn apply(&self, ctx: &mut ApplyContext<'_>) -> Result<()> {
  ctx.compilation_hooks.process_assets.tap(process_assets::new(self));
  ctx.compilation_hooks.emit.tap(emit::new(self));
  Ok(())
 }
}
```

## Hook Usage

### Accessing Compilation Data

```rust
#[plugin_hook(CompilationProcessAssets for MyPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
 // Access chunks
 for chunk in compilation.chunk_by_ukey.values() {
  // Work with chunk
 }

 // Access modules
 for module in compilation.module_graph.modules().values() {
  // Work with module
 }

 // Access assets
 compilation.assets_mut().iter_mut().for_each(|(name, asset)| {
  // Modify assets
 });

 Ok(())
}
```

### Updating Assets

```rust
#[plugin_hook(CompilationProcessAssets for MyPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
 compilation.update_asset("filename.js", |old, info| {
  let new_source = self.transform(old);
  Ok((new_source, info))
 })?;
 Ok(())
}
```

### Using Logger

```rust
#[plugin_hook(CompilationProcessAssets for MyPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
 let logger = compilation.get_logger("rspack.MyPlugin");
 let start = logger.time("operation");
 // ... do work ...
 logger.time_end(start);
 logger.info("Processing assets");
 Ok(())
}
```

## Error Handling

### Propagating Errors

```rust
use rspack_error::Result;

async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
 let result = some_fallible_operation().await?;
 Ok(())
}
```

### Adding Context to Errors

```rust
use rspack_error::{Result, Error};

async fn process_file(&self, filename: &str) -> Result<String> {
 some_operation()
  .await
  .map_err(|e| Error::internal_error(format!("Failed to process {}: {}", filename, e)))
}
```

### Batch Error Handling

```rust
use rspack_error::{BatchErrors, Result};

async fn process_multiple(&self, files: Vec<&str>) -> Result<()> {
 let mut errors = BatchErrors::default();
 for file in files {
  if let Err(e) = self.process_file(file).await {
   errors.push(e);
  }
 }
 errors.into_result()
}
```

## Async Operations

### Async Function in Trait

```rust
use futures::future::BoxFuture;

pub type MyAsyncFn = Box<
 dyn for<'a> Fn(MyContext<'a>) -> BoxFuture<'a, Result<String>> + Sync + Send
>;
```

### Parallel Processing

```rust
use futures::future::join_all;

async fn process_multiple(&self, items: Vec<Item>) -> Result<Vec<Result>>> {
 let futures: Vec<_> = items
  .into_iter()
  .map(|item| self.process_item(item))
  .collect();
 join_all(futures).await
}
```

## Testing Patterns

### Rust Unit Test

```rust
#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_plugin_creation() {
  let options = MyPluginOptions {
   option1: "test".to_string(),
   option2: Some(true),
  };
  let plugin = MyPlugin::new(options);
  assert_eq!(plugin.name(), "rspack.MyPlugin");
 }
}
```

### JavaScript Integration Test

```javascript
// tests/rspack-test/configCases/my-plugin/index.js
import rspack from '@rspack/core';

it('should process assets correctly', async () => {
  const compiler = rspack({
    entry: './index.js',
    plugins: [new MyPlugin({ option1: 'test' })],
  });

  const stats = await new Promise((resolve, reject) => {
    compiler.run((err, stats) => {
      if (err) reject(err);
      else resolve(stats);
    });
  });

  expect(stats.hasErrors()).toBe(false);
});
```

## Loader Implementation

### Basic Loader Structure

```rust
use rspack_core::{LoaderContext, LoaderResult};
use rspack_error::Result;

pub async fn my_loader(
 loader_context: &mut LoaderContext<'_>,
 content: &[u8],
) -> Result<LoaderResult> {
 let transformed = transform_content(content)?;
 Ok(LoaderResult::ok(transformed.into()))
}
```

### Loader with Options

```rust
#[derive(Debug, Deserialize)]
pub struct MyLoaderOptions {
 pub option1: String,
 pub option2: Option<bool>,
}

pub async fn my_loader(
 loader_context: &mut LoaderContext<'_>,
 content: &[u8],
) -> Result<LoaderResult> {
 let options: MyLoaderOptions = serde_json::from_str(
  loader_context.options.as_str().unwrap_or("{}")
 )?;
 let transformed = transform_with_options(content, &options)?;
 Ok(LoaderResult::ok(transformed.into()))
}
```

## Configuration Options

### Defining Options

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyPluginOptions {
 /// Description of option1
 pub option1: String,

 /// Optional description of option2
 #[serde(default)]
 pub option2: Option<bool>,

 /// Default value for option3
 #[serde(default = "default_option3")]
 pub option3: i32,
}

fn default_option3() -> i32 {
 10
}
```

### TypeScript Options Type

```typescript
export interface MyPluginOptions {
  /**
   * Description of option1
   */
  option1: string;

  /**
   * Optional description of option2
   */
  option2?: boolean;

  /**
   * Default value is 10
   * @default 10
   */
  option3?: number;
}
```

## Asset Processing Patterns

### Reading and Modifying Assets

```rust
#[plugin_hook(CompilationProcessAssets for MyPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
 for (filename, asset) in compilation.assets() {
  if self.should_process(filename) {
   let content = asset.source().to_string();
   let modified = self.transform(&content)?;
   compilation.update_asset(filename.as_str(), |old, info| {
    Ok((RawStringSource::from(modified).boxed(), info))
   })?;
  }
 }
 Ok(())
}
```

### Creating New Assets

```rust
#[plugin_hook(CompilationProcessAssets for MyPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
 let new_asset = RawStringSource::from("new content").boxed();
 compilation.emit_asset("new-file.js".to_string(), new_asset);
 Ok(())
}
```

## String Processing

### Using CowUtils

```rust
use cow_utils::CowUtils;

fn process_string(input: &str) -> String {
 let result = input
  .cow_to_lowercase()
  .cow_replace("old", "new");
 result.into_owned()
}
```

### Regex Patterns

```rust
use std::sync::LazyLock;
use regex::Regex;

static MY_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
 Regex::new(r"pattern").expect("invalid regexp")
});

fn match_pattern(input: &str) -> bool {
 MY_PATTERN.is_match(input)
}
```

## Performance Optimization Patterns

### Avoiding Allocations

```rust
// Prefer &str over String when possible
fn process<'a>(input: &'a str) -> &'a str {
 input
}

// Use Cow<str> for conditional ownership
use std::borrow::Cow;

fn maybe_owned(input: &str) -> Cow<str> {
 if condition {
  Cow::Owned(input.to_uppercase())
 } else {
  Cow::Borrowed(input)
 }
}
```

### Efficient Iteration

```rust
let results: Vec<_> = items
 .iter()
 .filter(|item| self.should_process(item))
 .map(|item| self.process(item))
 .collect();
```

## Common Utilities

### Path Handling

```rust
use rspack_core::{Filename, PathData};

let filename = compilation.get_path(
 &Filename::from("template.[hash].js"),
 PathData::default()
  .hash(&hash)
  .chunk_id_optional(chunk.id().map(|id| id.as_str()))
  .filename(file),
).await?;
```

### Hash Generation

```rust
let hash = compilation
 .hash
 .as_ref()
 .expect("should have compilation.hash")
 .encoded()
 .to_owned();
```

## Resources

- See [CODE_STYLE.md](./CODE_STYLE.md) for coding standards
- See [AGENTS.md](../AGENTS.md) for development workflow
- See existing plugins in `crates/rspack_plugin_*/` for examples
- See existing loaders in `crates/rspack_loader_*/` for examples

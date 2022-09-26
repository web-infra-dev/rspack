// use std::collections::HashMap;

// use pathdiff::diff_paths;
// use rspack_core::{
//   rspack_sources::{MapOptions, RawSource, SourceExt},
//   Plugin, PluginContext, PluginProcessAssetsOutput, ProcessAssetsArgs,
// };

// #[derive(Debug)]
// pub struct DevtoolPlugin {}

// #[async_trait::async_trait]
// impl Plugin for DevtoolPlugin {
//   fn name(&self) -> &'static str {
//     "devtool"
//   }

//   async fn process_assets(
//     &self,
//     _ctx: PluginContext,
//     args: ProcessAssetsArgs<'_>,
//   ) -> PluginProcessAssetsOutput {
//     if !args.compilation.options.devtool {
//       return Ok(());
//     }
//     let mut maps = HashMap::new();
//     for (filename, asset) in &args.compilation.assets {
//       if let Some(map) = asset.map(&MapOptions::default()) {
//         maps.insert(filename.to_owned(), map);
//       }
//     }
//     for (filename, mut map) in maps {
//       map.set_file(Some(filename.clone()));
//       for source in map.sources_mut() {
//         let uri = &source[1..source.len() - 1]; // remove '<' and '>' in <uri>
//         *source = if let Some(relative_path) = diff_paths(uri, &args.compilation.options.context) {
//           relative_path.display().to_string()
//         } else {
//           uri.to_owned()
//         };
//       }
//       let map = map
//         .to_json()
//         .map_err(|e| rspack_error::Error::InternalError(e.to_string()))?;
//       args
//         .compilation
//         .emit_asset(filename + ".map", RawSource::from(map).boxed());
//     }
//     Ok(())
//   }
// }

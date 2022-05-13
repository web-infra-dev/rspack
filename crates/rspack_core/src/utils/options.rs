use crate::{BundleOptions, NormalizedBundleOptions};

pub fn normalize_bundle_options(options: BundleOptions) -> NormalizedBundleOptions {
  NormalizedBundleOptions {
    resolve: options.resolve,
    react: options.react,
    loader: options.loader.unwrap_or_default(),
    mode: options.mode,
    entries: options.entries,
    minify: options.minify,
    outdir: options.outdir,
    entry_filename: options.entry_file_names,
    chunk_filename: options
      .chunk_filename
      .unwrap_or("chunk-[contenthash].js".to_string()),
    code_splitting: options.code_splitting,
    root: options.root,
    source_map: options.source_map,
  }
}

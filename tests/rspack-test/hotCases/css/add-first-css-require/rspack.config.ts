import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    chunkLoading: 'require',
    chunkFormat: 'commonjs',
    // The stale (pre-apply) runtime resolves the newly added stylesheet, so
    // the css filename must be derivable from the chunk id alone.
    cssFilename: '[name].css',
    cssChunkFilename: '[name].css',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
      {
        // Inline CSS enables the CSS runtime with the require chunk loader.
        test: /inline\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'style',
        },
      },
    ],
  },
});

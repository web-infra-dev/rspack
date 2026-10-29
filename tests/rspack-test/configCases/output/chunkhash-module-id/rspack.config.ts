import { defineConfig } from '@rspack/cli';

// The two builds differ only in module ids. The async chunk's module does not
// reference its own id, so only the chunk's module map key changes.
export default defineConfig([
  {
    mode: 'production',
    output: {
      filename: 'bundle0.js',
      chunkFilename: 'chunk0.[chunkhash].js',
    },
    optimization: {
      chunkIds: 'named',
      moduleIds: 'named',
      concatenateModules: false,
      minimize: false,
    },
  },
  {
    mode: 'production',
    output: {
      filename: 'bundle1.js',
      chunkFilename: 'chunk1.[chunkhash].js',
    },
    optimization: {
      chunkIds: 'named',
      moduleIds: 'deterministic',
      concatenateModules: false,
      minimize: false,
    },
  },
]);

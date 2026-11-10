import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  externalsType: 'module',
  externals: {
    './x.js': './x.js',
    './y.js': './y.js',
  },
  optimization: {
    minimize: false,
    moduleIds: 'named',
    chunkIds: 'named',
    runtimeChunk: false,
  },
});

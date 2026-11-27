import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  externalsType: 'module',
  externals: {
    './ext.mjs': './ext.mjs',
    './side.mjs': './side.mjs',
  },
  optimization: {
    minimize: false,
    moduleIds: 'named',
    chunkIds: 'named',
    runtimeChunk: false,
  },
});

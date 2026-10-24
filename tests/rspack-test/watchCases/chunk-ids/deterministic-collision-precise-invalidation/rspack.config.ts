import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: ['./index.js', './trigger.js'],
  mode: 'development',
  cache: false,
  incremental: 'advance',
  devtool: false,
  optimization: {
    moduleIds: 'named',
    chunkIds: 'deterministic',
    concatenateModules: false,
    inlineExports: false,
    mangleExports: false,
    usedExports: false,
    splitChunks: false,
    minimize: false,
  },
});

import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  cache: false,
  incremental: { silent: true },
  devtool: false,
  optimization: {
    moduleIds: 'deterministic',
    chunkIds: 'named',
    concatenateModules: false,
    inlineExports: false,
    mangleExports: false,
    usedExports: false,
    splitChunks: false,
    minimize: false,
  },
});

import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  optimization: {
    splitChunks: {
      minSize: {},
      maxSize: {
        webassembly: 500,
      },
    },
  },
  stats: {
    assets: true,
    chunks: true,
    chunkModules: true,
    dependentModules: true,
    modules: true,
  },
  experiments: {
    asyncWebAssembly: true,
  },
});

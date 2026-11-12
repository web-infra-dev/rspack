import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  externals: {
    pkg: {
      root: 'pkg',
      commonjs: 'pkg',
      commonjs2: 'pkg',
      amd: 'pkg',
    },
  },
  optimization: {
    concatenateModules: true,
  },
});

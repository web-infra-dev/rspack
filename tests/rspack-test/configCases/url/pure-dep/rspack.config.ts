import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  output: {
    assetModuleFilename: '[path][name][ext]',
  },
  optimization: {
    minimize: false,
    innerGraph: true,
  },
  module: {
    parser: {
      javascript: {
        // this is always true in rspack
        // dynamicUrl: true
      },
    },
  },
});

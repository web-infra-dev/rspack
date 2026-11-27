import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.wat$/,
        loader: 'wast-loader',
        type: 'webassembly/async',
      },
    ],
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    concatenateModules: false,
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: {
          test: /add\.wat$/,
          name: 'shared',
          minChunks: 2,
          enforce: true,
        },
      },
    },
  },
  experiments: {
    asyncWebAssembly: true,
  },
});

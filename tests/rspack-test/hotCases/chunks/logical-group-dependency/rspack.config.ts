import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    moduleIds: 'named',
    chunkIds: 'named',
    concatenateModules: false,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        default: false,
        defaultVendors: false,
        dependency: {
          test: /dependency\.js$/,
          name: 'dependency',
          enforce: true,
        },
      },
    },
  },
});

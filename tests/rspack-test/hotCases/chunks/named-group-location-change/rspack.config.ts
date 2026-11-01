import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    concatenateModules: false,
    splitChunks: {
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: {
          test: /[\\/]dep\.js$/,
          name: 'shared',
          chunks: 'all',
          enforce: true,
          minSize: 0,
        },
      },
    },
  },
});

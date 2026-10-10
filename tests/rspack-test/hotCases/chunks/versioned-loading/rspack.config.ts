import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: { clean: true, chunkFilename: '[name].js' },
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
          test: /[\\/](before|after|latest)\.js$/,
          name: (module) =>
            module.nameForCondition()!.split(/[\\/]/).pop()!.slice(0, -3),
          enforce: true,
        },
      },
    },
  },
});

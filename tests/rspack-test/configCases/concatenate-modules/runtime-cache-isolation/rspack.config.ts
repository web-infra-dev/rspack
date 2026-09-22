import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    a: './a.js',
    b: './b.js',
  },
  output: {
    filename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        sideEffects: false,
      },
    ],
  },
  optimization: {
    concatenateModules: true,
    innerGraph: true,
    minimize: false,
    sideEffects: true,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        shared: {
          enforce: true,
          name: 'shared',
          test: /root-shared/,
        },
      },
    },
    usedExports: true,
  },
  stats: {
    modules: true,
    nestedModules: true,
    optimizationBailout: true,
  },
});

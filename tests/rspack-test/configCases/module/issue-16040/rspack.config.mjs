import { CssExtractRspackPlugin } from '@rspack/core';
export default {
  mode: 'production',
  devtool: false,
  experiments: {
    css: false,
  },
  output: {
    publicPath: '/',
    filename: '[name].mjs',
    chunkFilename: '[name].chunk.js',
    assetModuleFilename: '[hash][ext][query]',
    module: true,
    library: { type: 'module' },
    chunkFormat: 'module',
    chunkLoading: 'import',
    environment: {
      dynamicImport: true,
      module: true,
    },
  },

  module: {
    rules: [
      {
        type: 'javascript/auto',
        test: /\.css$/i,
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },

  plugins: [
    new CssExtractRspackPlugin({
      filename: 'style.css',
      chunkFilename: '[id].css',
    }),
  ],

  optimization: {
    splitChunks: {
      chunks: 'all',

      cacheGroups: {
        style: {
          name: 'style',
          type: 'css/mini-extract',
          chunks: 'all',
          enforce: true,
        },

        defaultVendors: {
          name: 'vendor',
          test: /[\\/]node_modules[\\/]/,
          priority: -10,
          chunks: 'initial',
          reuseExistingChunk: true,
        },

        default: {
          minChunks: 2,
          priority: -20,
          reuseExistingChunk: true,
        },
      },
    },

    runtimeChunk: {
      name: 'runtime',
    },

    // currently Webpack has bugs when setting concatenateModules to true while produce ES Module output.
    // concatenateModules: false,

    minimize: false,
  },
};

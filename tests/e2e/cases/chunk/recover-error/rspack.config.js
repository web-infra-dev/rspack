const { rspack } = require('@rspack/core');
const ReactRefreshPlugin = require('@rspack/plugin-react-refresh');

module.exports = {
  mode: 'development',
  context: __dirname,
  entry: {
    main: './src/main.jsx',
  },
  devtool: false,
  resolve: {
    extensions: ['...', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.(jsx?|tsx?)$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                parser: {
                  syntax: 'typescript',
                  tsx: true,
                },
                transform: {
                  react: {
                    runtime: 'automatic',
                    development: true,
                    refresh: true,
                  },
                },
              },
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/index.html' }),
    new ReactRefreshPlugin(),
  ],
  optimization: {
    runtimeChunk: {
      name: 'builder-runtime',
    },
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        react: {
          name: 'lib-react',
          test: /node_modules[\\/](?:react|react-dom|scheduler|react-refresh|@rspack[\\/]plugin-react-refresh)[\\/]/,
          priority: 0,
        },
      },
    },
  },
  devServer: {
    hot: true,
    devMiddleware: {
      writeToDisk: true,
    },
  },
};

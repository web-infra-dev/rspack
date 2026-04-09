const path = require('node:path');
const { rspack } = require('@rspack/core');
const { ReactRefreshRspackPlugin } = require('@rspack/plugin-react-refresh');

module.exports = {
  context: __dirname,
  mode: 'development',
  entry: {
    main: './src/main.jsx',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/index.html' }),
    new ReactRefreshRspackPlugin(),
    new rspack.CssExtractRspackPlugin(),
  ],
  resolve: {
    extensions: ['...', '.ts', '.tsx', '.jsx'],
  },
  experiments: {
    css: false,
  },
  module: {
    rules: [
      {
        test: /\.jsx$/,
        use: {
          loader: 'builtin:swc-loader',
          options: {
            detectSyntax: 'auto',
            jsc: {
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
      },
      {
        type: 'javascript/auto',
        test: /\.css$/,
        use: [
          rspack.CssExtractRspackPlugin.loader,
          'css-loader',
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: {
                  tailwindcss: {
                    config: path.join(__dirname, './tailwind.config.js'),
                  },
                },
              },
            },
          },
        ],
      },
    ],
  },
};

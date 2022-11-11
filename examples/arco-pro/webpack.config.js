const path = require('path');
const ReactRefreshWebpackPlugin = require('@pmmmwh/react-refresh-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

const isDevelopment = process.env.NODE_ENV !== 'production';

/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  context: __dirname,
  entry: { main: './src/index.tsx' },
  devServer: {
    port: 5556
  },
  devtool: 'source-map',
  mode: isDevelopment ? 'development' : 'production',
  module: {
    rules: [
      {
        test: /\.[jt]sx?$/,
        exclude: /node_modules/,
        use: [
          {
            loader: 'swc-loader',
            options: {
              jsc: {
                transform: {
                  react: {
                    development: isDevelopment,
                    refresh: isDevelopment,
                  },
                },
              },
            },
          },
        ],
      },
      {
        test: /\.svg$/i,
        issuer: /\.[jt]sx?$/,
        use: ['@svgr/webpack'],
      },
      {
        test: /\.css$/i,
        use: [isDevelopment ? 'style-loader' : MiniCssExtractPlugin.loader, "css-loader"],
      },
      {
        test: /\.less$/i,
        use: [isDevelopment ? 'style-loader' : MiniCssExtractPlugin.loader, "css-loader", "less-loader"],
      },
    ],
  },
  plugins: [
    isDevelopment && new ReactRefreshWebpackPlugin(),
    new HtmlWebpackPlugin({
      template: './index.html',
    }),
    !isDevelopment && new MiniCssExtractPlugin(),
  ].filter(Boolean),
  resolve: {
    alias: { '@': path.resolve(__dirname, 'src') },
    extensions: ['.js', '.jsx', '.ts', '.tsx'],
  },
  output: {
    publicPath: '/'
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  infrastructureLogging: {
    debug: false
  },
}
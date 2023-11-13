const { defineConfig } = require('@rspack/cli')
const { HtmlRspackPlugin } = require('@rspack/core')
const { VueLoaderPlugin } = require('vue-loader')

const config = defineConfig({
  context: __dirname,

  entry: { main: './src/main.ts' },

  devServer: {
    open: true,
    historyApiFallback: true,
  },

  experiments: { css: true },

  plugins: [
    new VueLoaderPlugin(),
    new HtmlRspackPlugin({
      template: './index.html',
      title: 'Vue2 + TSX + Rspack',
      favicon: './src/assets/logo.png',
    }),
  ],

  module: {
    rules: [
      {
        test: /\.vue$/,
        use: {
          loader: 'vue-loader',
          options: {
            experimentalInlineMatchResource: true,
            compilerOptions: { preserveWhitespace: false },
          },
        },
      },
      {
        test: /\.ts$/,
        loader: 'builtin:swc-loader',
        options: {
          sourceMap: true,
          jsc: {
            parser: { syntax: 'typescript' },
          },
        },
        type: 'javascript/auto',
      },
      {
        test: /\.[jt]sx$/,
        use: {
          loader: 'babel-loader',
          options: {
            presets: [
              ['@babel/preset-typescript', { isTSX: true, allExtensions: true }],
              ['@vue/babel-preset-jsx', { compositionAPI: true }],
            ],
          },
        },
      },
      {
        test: /\.less$/,
        use: 'less-loader',
        type: 'css',
      },
    ],
  },
})

module.exports = config

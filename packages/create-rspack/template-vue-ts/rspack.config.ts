import { defineConfig } from '@rspack/cli';
import {
  type RspackPluginFunction,
  rspack,
  type SwcLoaderOptions,
} from '@rspack/core';
import { VueLoaderPlugin } from 'rspack-vue-loader';

export default defineConfig({
  entry: {
    main: './src/main.ts',
  },
  target: ['browserslist:last 2 versions, > 0.2%, not dead, Firefox ESR'],
  resolve: {
    extensions: ['...', '.ts', '.vue'],
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
      {
        test: /\.vue$/,
        loader: 'rspack-vue-loader',
        options: {
          experimentalInlineMatchResource: true,
        },
      },
      {
        test: /\.(js|ts)$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                parser: {
                  syntax: 'typescript',
                },
              },
            } satisfies SwcLoaderOptions,
          },
        ],
      },
      {
        test: /\.svg/,
        type: 'asset/resource',
      },
    ],
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './index.html',
    }),
    new rspack.DefinePlugin({
      __VUE_OPTIONS_API__: true,
      __VUE_PROD_DEVTOOLS__: false,
    }),
    new VueLoaderPlugin() as RspackPluginFunction,
  ],
});

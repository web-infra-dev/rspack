import { defineConfig } from '@rspack/cli';
import { rspack, type SwcLoaderOptions } from '@rspack/core';

export default defineConfig({
  entry: {
    main: './src/index.ts',
  },
  target: ['browserslist:last 2 versions, > 0.2%, not dead, Firefox ESR'],
  resolve: {
    extensions: ['...', '.ts'],
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
      {
        test: /\.svg$/,
        type: 'asset',
      },
      {
        test: /\.js$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                parser: {
                  syntax: 'ecmascript',
                },
              },
            } satisfies SwcLoaderOptions,
          },
        ],
      },
      {
        test: /\.ts$/,
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
    ],
  },
  plugins: [new rspack.HtmlRspackPlugin({ template: './index.html' })],
});

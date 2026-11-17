import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /index\.js$/,
        oneOf: [
          {
            resourceQuery: '?json',
            loader: './mark-loader.mjs',
            options: { raw: '{"unchangedJsonRule":true}' },
          },
          {
            loader: './mark-loader.mjs',
            options: { value: 'compiled' },
          },
        ],
      },
      {
        test: /\.css$/,
        type: 'asset/resource',
      },
      {
        with: { type: 'text' },
        include: /intentional\.js/,
        loader: './mark-loader.mjs',
        options: { value: 'kept' },
      },
      {
        test: /\.js\.text$/,
        resourceQuery: '?rewritten',
        loader: './mark-loader.mjs',
        options: { value: 'rewritten' },
      },
    ],
  },
});

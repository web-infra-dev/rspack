import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        resource: /abc\.js$/,
        use: [
          {
            loader: './loader.mjs',
            options: 'a',
          },
          {
            loader: './loader.mjs',
            options: 'b',
          },
        ],
      },
      {
        resource: /def\.js$/,
        use: [
          {
            loader: './loader.mjs',
            options: 'c',
          },
          {
            loader: './loader.mjs',
            options: 'd',
          },
        ],
      },
    ],
  },
});

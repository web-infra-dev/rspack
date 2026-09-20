import { defineConfig } from '@rspack/cli';

export default defineConfig({
  resolveLoader: {
    alias: {
      'my-loader': './loader.mjs?query=alias',
    },
  },
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: {
          loader: './loader.mjs?query=a',
        },
      },
      {
        test: /b\.js$/,
        use: {
          loader: 'my-loader',
        },
      },
      {
        test: /c\.js$/,
        use: {
          loader: 'my-loader?query=c',
        },
      },
      {
        test: /d\.js$/,
        use: {
          loader: './loader.mjs',
          options: 'query=d',
        },
      },
      {
        test: /e\.js$/,
        use: {
          loader: './loader.mjs?query=e',
          options: 'query=options-e',
        },
      },
      {
        test: /f\.js$/,
        use: {
          loader: './loader.mjs?query=f',
          options: {
            query: 'options-object-f',
          },
        },
      },
      {
        test: /g\.js$/,
        use: {
          loader: 'my-loader',
          options: {
            query: 'options-object-g',
          },
        },
      },
      {
        test: /h\.js$/,
        use: {
          loader: 'my-loader?query=h',
          options: {
            query: 'options-object-h',
          },
        },
      },
    ],
  },
});

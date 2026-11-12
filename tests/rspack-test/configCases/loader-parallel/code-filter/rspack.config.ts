import { defineConfig } from '@rspack/cli';

const observer = {
  loader: './observer-loader.mjs',
  parallel: true,
  options: {},
};

const producer = {
  loader: './producer-loader.mjs',
  parallel: true,
  options: {},
};

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /hit\.js$/,
        use: [
          observer,
          {
            loader: './transform-loader.mjs',
            parallel: true,
            options: { value: 'hit' },
            codeFilter: 'generated:marker',
          },
          producer,
        ],
      },
      {
        test: /miss\.js$/,
        use: [
          observer,
          {
            loader: './transform-loader.mjs',
            parallel: true,
            options: { value: 'should-not-run' },
            codeFilter: /not present/,
          },
          producer,
        ],
      },
      {
        test: /invalid\.js$/,
        use: [
          {
            loader: './invalid-finalizer-loader.mjs',
            parallel: true,
            options: {},
          },
          {
            loader: './invalid-transform-loader.mjs',
            parallel: true,
            options: {},
            codeFilter: /(?:)/,
          },
          {
            loader: './invalid-producer-loader.mjs',
            parallel: true,
            options: {},
          },
        ],
      },
    ],
  },
});

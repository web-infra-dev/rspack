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
            codeFilter: /generated:marker/g,
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
            codeFilter: 'not present',
          },
          producer,
        ],
      },
    ],
  },
});

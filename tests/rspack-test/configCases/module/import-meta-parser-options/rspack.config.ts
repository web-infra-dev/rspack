import { defineConfig } from '@rspack/cli';

const importMeta = Object.assign(
  Object.create({
    get url() {
      throw new Error('Inherited importMeta properties must not be read');
    },
  }),
  {
    ignored: Object.defineProperty([], 0, {
      get() {
        throw new Error('Non-boolean importMeta values must not be traversed');
      },
    }),
  },
);

export default defineConfig({
  mode: 'development',
  devtool: false,
  target: 'node',
  output: {
    module: true,
    chunkFormat: 'module',
  },
  module: {
    parser: {
      javascript: { importMeta },
    },
    rules: [
      {
        test: /empty-options\.js$/,
        parser: {
          importMeta,
        },
      },
      {
        test: /disabled-fields\.js$/,
        parser: {
          importMeta: {
            dirname: false,
            filename: false,
            main: false,
            url: false,
            webpack: false,
            webpackContext: false,
            glob: false,
            webpackHot: false,
          },
        },
      },
    ],
  },
});

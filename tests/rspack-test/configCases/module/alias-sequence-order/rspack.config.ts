import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        resolve: {
          alias: {
            'foo/bar': './exist',
          },
        },
      },
      {
        resolve: {
          alias: {
            foo: './not-exist',
          },
        },
      },
    ],
  },
});

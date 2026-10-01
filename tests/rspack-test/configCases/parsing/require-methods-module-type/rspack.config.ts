import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /cjs\.js$/,
        type: 'javascript/dynamic',
      },
      {
        test: /esm\.js$/,
        type: 'javascript/esm',
      },
    ],
  },
});

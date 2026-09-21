import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  module: {
    rules: [
      {
        test: /\.js$/,
        type: 'javascript/esm',
        use: 'builtin:swc-loader',
      },
    ],
  },
});

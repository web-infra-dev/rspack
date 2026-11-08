import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index.ts',
  resolve: {
    extensions: ['.ts', '.js'],
  },
  optimization: {
    concatenateModules: true,
  },
  module: {
    rules: [
      {
        test: /\.(j|t)s$/,
        loader: 'builtin:swc-loader',
        options: {},
        type: 'javascript/auto',
      },
    ],
  },
});

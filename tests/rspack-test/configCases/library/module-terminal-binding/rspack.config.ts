import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'web',
  optimization: {
    minimize: false,
  },
  output: {
    filename: '[name].mjs',
    library: {
      type: 'module',
    },
  },
});

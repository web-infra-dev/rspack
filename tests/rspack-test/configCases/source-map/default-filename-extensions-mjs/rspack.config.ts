import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    filename: 'bundle0.mjs',
    module: true,
  },
  devtool: 'source-map',
});

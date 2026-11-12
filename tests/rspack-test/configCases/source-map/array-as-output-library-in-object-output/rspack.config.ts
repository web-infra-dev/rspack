import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: 'source-map',
  output: {
    library: { type: 'umd' },
  },
});

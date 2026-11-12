import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  output: {
    library: {
      type: 'commonjs',
    },
  },
  optimization: {
    minimize: false,
  },
});

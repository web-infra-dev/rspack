import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  output: {
    library: {
      type: 'commonjs',
    },
  },
});

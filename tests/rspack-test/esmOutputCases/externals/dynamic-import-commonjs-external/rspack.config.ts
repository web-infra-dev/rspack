import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    os: 'commonjs os',
  },
});

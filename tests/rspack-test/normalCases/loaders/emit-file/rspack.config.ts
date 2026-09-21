import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    './extra-file.js': 'commonjs ./extra-file.js',
  },
});

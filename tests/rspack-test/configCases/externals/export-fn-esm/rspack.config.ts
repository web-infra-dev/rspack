import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    module: 'commonjs module',
    fs: 'commonjs fs',
  },
});

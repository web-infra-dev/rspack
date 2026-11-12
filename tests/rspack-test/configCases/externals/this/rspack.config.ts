import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    concatenateModules: true,
  },
  externals: {
    external: 'this EXTERNAL_TEST_GLOBAL',
  },
});

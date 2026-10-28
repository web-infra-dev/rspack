import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    external: 'global EXTERNAL_TEST_GLOBAL',
  },
});

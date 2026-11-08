import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  cache: true,
  module: {},
  externals: {
    external: 'var 123',
  },
});

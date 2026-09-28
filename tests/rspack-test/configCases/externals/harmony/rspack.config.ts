import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    external: "var 'abc'",
  },
});

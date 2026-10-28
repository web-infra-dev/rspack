import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: { type: 'umd2' },
  },
  externals: {
    external0: 'external0',
    external1: "var 'abc'",
  },
  node: {
    __dirname: false,
    __filename: false,
  },
});

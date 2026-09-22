import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: { type: 'umd' },
  },
  externals: {
    external: {
      root: ['a', 'b'],
      commonjs: 'a',
      commonjs2: 'a',
      amd: 'a',
    },
  },
});

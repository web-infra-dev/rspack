import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry() {
    return {
      a: { import: './a' },
      b: { import: ['./b'] },
    };
  },
  output: {
    filename: '[name].js',
  },
});

import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry() {
    return {
      a: './a',
      b: ['./b'],
    };
  },
  output: {
    filename: '[name].js',
  },
});

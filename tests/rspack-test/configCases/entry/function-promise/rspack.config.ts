import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry() {
    return Promise.resolve({
      a: './a',
      b: ['./b'],
    });
  },
  output: {
    filename: '[name].js',
  },
});

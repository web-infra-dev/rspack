import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry() {
    return {
      a: './a',
    };
  },
  output: {
    filename: '[name].js',
  },
});

import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    a1: './a',
    b1: {
      runtime: 'a1',
      import: './b',
    },
  },
});

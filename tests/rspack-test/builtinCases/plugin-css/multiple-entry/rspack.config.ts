import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    'main-one': {
      import: ['./index-one.js'],
    },
    'main-two': {
      import: ['./index-two.js'],
    },
  },
});

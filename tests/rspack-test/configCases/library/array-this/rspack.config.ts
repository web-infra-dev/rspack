import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  output: {
    library: { type: 'this', name: ['a', 'b'] },
    environment: {
      arrowFunction: false,
    },
  },
});

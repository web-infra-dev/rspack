import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  output: {
    library: { type: 'window', name: ['a', 'b'] },
  },
});

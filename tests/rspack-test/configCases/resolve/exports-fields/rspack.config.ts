import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  resolve: {
    exportsFields: ['a', 'b'],
  },
});

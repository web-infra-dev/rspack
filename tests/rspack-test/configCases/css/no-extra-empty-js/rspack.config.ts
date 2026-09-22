import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  module: {
    rules: [{ test: /\.css$/, type: 'css/auto' }],
  },
});

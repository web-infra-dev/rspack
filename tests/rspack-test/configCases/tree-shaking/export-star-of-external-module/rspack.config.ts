import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [],
  },

  optimization: {
    sideEffects: true,
  },
  externalsPresets: {
    node: true,
  },
  externals: {
    'react-router-dom': 'Buffer',
  },
});

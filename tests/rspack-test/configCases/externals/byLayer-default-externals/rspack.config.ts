import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externalsPresets: { node: false },
  externals: {
    base: 'var 42',
    value: 'var 42',
    byLayer: {
      app: {},
      default: { value: 'var 43' },
    },
  },
  module: { rules: [{ test: /app\.js$/, layer: 'app' }] },
});

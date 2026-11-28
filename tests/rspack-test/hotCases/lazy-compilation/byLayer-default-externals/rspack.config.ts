import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: { library: { name: 'TestLib', type: 'umd' } },
  externalsType: 'umd',
  externals: {
    fs: 'fs',
    byLayer: { default: { path: 'path' } },
  },
  module: { rules: [{ test: /module\.js$/, layer: 'other-layer' }] },
  lazyCompilation: { entries: false },
});

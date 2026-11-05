import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: { library: { name: 'TestLib', type: 'umd' } },
  externalsType: 'umd',
  externals: {
    fs: { root: 'fs', commonjs: 'fs', commonjs2: 'fs', amd: 'fs' },
    byLayer: { 'app-layer': { fs: { commonjs: 'fs' }, path: 'path' } },
  },
  module: { rules: [{ test: /module\.js$/, layer: 'app-layer' }] },
  lazyCompilation: { entries: false },
});

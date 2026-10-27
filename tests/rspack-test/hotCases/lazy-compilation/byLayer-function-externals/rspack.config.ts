import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: { library: { name: 'TestLib', type: 'umd' } },
  externalsType: 'umd',
  externals: {
    fs: { root: 'fs' },
    byLayer: (layer) => ({
      fs: { commonjs: 'fs', commonjs2: 'fs', amd: 'fs' },
      path: layer === 'app-layer' ? 'path' : false,
      './module': layer === 'app-layer' ? 'var ({})' : false,
    }),
  },
  module: { rules: [{ test: /module\.js$/, layer: 'app-layer' }] },
  lazyCompilation: { entries: false },
});

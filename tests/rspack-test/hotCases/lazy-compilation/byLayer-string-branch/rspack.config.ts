import { createRequire } from 'node:module';
import { defineConfig } from '@rspack/cli';

const require = createRequire(import.meta.url);

export default defineConfig(
  [false, true].map((dynamic) => ({
    output: { library: { name: 'TestLib', type: 'umd' } },
    externalsPresets: { node: false },
    externalsType: 'umd',
    externals: [
      { 'node:module': 'commonjs node:module' },
      { byLayer: dynamic ? () => 'path' : { app: 'path' } },
    ],
    module: { rules: [{ test: /module\.js$/, layer: 'app' }] },
    lazyCompilation: {
      entries: false,
      client: require.resolve('@rspack/core/hot/lazy-compilation-node'),
    },
  })),
);

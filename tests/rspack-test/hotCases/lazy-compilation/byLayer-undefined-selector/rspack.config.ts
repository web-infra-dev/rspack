import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: { library: { name: 'TestLib', type: 'umd' } },
  externalsType: 'umd',
  externals: { bridge: 'path', byLayer: undefined },
  lazyCompilation: { entries: false },
});

import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  optimization: {
    concatenateModules: false,
    inlineExports: true,
    moduleIds: 'named',
    sideEffects: true,
    usedExports: true,
  },
});

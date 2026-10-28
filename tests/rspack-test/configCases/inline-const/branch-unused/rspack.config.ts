import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    inlineExports: true,
    moduleIds: 'named',
    usedExports: true,
  },
});

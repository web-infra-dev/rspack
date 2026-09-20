import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  node: {
    __dirname: false,
  },
  optimization: {
    emitOnErrors: false,
  },
});

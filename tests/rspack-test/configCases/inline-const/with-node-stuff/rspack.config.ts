import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  target: 'web',
  mode: 'production',
  node: {
    global: true,
    __filename: false,
  },
  optimization: {
    minimize: false,
    inlineExports: true,
  },
});

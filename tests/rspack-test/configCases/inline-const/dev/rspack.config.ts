import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  mode: 'development',
  output: {
    pathinfo: false,
  },
  optimization: {
    inlineExports: true,
  },
});

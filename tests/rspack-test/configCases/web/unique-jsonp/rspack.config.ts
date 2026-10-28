import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  output: {
    filename: '[name].js',
  },
  externals: {
    fs: 'commonjs2 fs',
  },
  node: {
    __filename: false,
    __dirname: false,
  },
});

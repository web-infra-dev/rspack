import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'none',
  target: 'node',
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    filename: '[name].js',
    workerPublicPath: '/workerPublicPath2/',
  },
});

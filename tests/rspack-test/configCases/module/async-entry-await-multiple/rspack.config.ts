import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  mode: 'development',
  devtool: false,
  entry: ['./first.js', './index.js'],
  output: {
    module: true,
    filename: 'bundle0.mjs',
  },
});

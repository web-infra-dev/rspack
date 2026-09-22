import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
    library: {
      type: 'system',
    },
  },
  optimization: {
    runtimeChunk: {
      name: 'runtime',
    },
  },
});

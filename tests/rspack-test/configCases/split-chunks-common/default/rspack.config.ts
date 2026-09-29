import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',

  entry: {
    main: './index',
  },
  target: 'node',
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
});

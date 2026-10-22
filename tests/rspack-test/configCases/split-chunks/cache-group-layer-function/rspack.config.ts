import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  entry: {
    main: {
      import: './index',
      layer: 'foo',
    },
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        bar: {
          name: 'bar',
          layer(layer) {
            return layer === 'bar';
          },
        },
        foo: {
          name: 'foo',
          layer(layer) {
            return layer === 'foo';
          },
        },
      },
    },
  },
});

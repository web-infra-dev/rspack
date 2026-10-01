import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        foo: {
          test: /\.js/,
          name(_module, chunks, cacheGroupKey) {
            expect(chunks.length).toBeGreaterThan(0);
            expect(cacheGroupKey).toBe('foo');
          },
        },
      },
    },
  },
});

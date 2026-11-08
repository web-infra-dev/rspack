import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  entry: './index.js',
  module: {
    rules: [
      {
        test: /index\.js$/,
        loader: './loader.mjs',
      },
    ],
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('Test', function (stats) {
          let s = stats.toJson({
            all: true,
          });
          expect(s.assets?.some((item) => item.name === 'a.txt')).toBeTruthy();
        });
      },
    }),
  ],
});

import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

let buildIndex = 0;

export default defineConfig({
  devtool: false,
  entry: {
    main: './index.js',
    vendors: './vendor.js',
    outside: './vendor.js',
    text: './vendor.js',
  },
  output: {
    filename: ({ chunk }) =>
      chunk!.name === 'text' ? 'main.txt' : '[name].js',
  },
  plugins: [
    new rspack.SourceMapDevToolPlugin({
      filename: '[file].map',
      test: /\.js$/,
      include: /^(main|vendors)\./,
      exclude: /^vendors\./,
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('CheckFilteredSourceMapCache', (stats) => {
          const compilation = stats.compilation;
          expect(compilation.getAsset('main.js.map')).toBeDefined();
          for (const filename of ['vendors.js', 'outside.js', 'main.txt']) {
            expect(compilation.getAsset(`${filename}.map`)).toBeUndefined();
            expect(
              compilation.getAsset(filename)!.source.source().toString(),
            ).not.toContain('sourceMappingURL');
          }
          const info = stats.toJson({ all: false, cacheInfo: true }).cacheInfo!;
          expect(info['|rspack.SourceMapDevToolPlugin']).toMatchObject({
            hits: buildIndex === 0 ? 0 : 1,
            misses: buildIndex === 0 ? 1 : 0,
            reused: buildIndex === 0 ? 0 : 1,
          });
          buildIndex++;
        });
      },
    }),
  ],
});

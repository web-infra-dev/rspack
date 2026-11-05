import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  entry: { main: { import: './index', layer: 'foo' } },
  output: { filename: '[name].js' },
  module: {
    rules: [{ test: /layer-rejected\.js$/, layer: 'bar' }],
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        bar: { name: 'bar', layer: /^bar$/ },
        foo: { name: 'foo', layer: /^foo$/ },
        layerAndTest: {
          name: 'layer-and-test',
          layer: /^foo$/,
          test: /[\\/]layer-/,
          priority: 10,
          enforce: true,
        },
        typeAndTest: {
          name: 'type-and-test',
          type: 'json',
          test: /[\\/]type-/,
          priority: 10,
          enforce: true,
        },
      },
    },
  },
  plugins: [
    definePlugin(function () {
      this.hooks.compilation.tap('CheckFilteredChunks', (compilation) => {
        compilation.hooks.afterSeal.tap('CheckFilteredChunks', () => {
          for (const [name, included, excluded] of [
            ['layer-and-test', 'layer-match.js', 'layer-rejected.js'],
            ['type-and-test', 'type-match.json', 'type-rejected.js'],
          ]) {
            const chunk = compilation.namedChunks.get(name);
            expect(chunk).toBeDefined();
            const identifiers = Array.from(
              compilation.chunkGraph.getChunkModulesIterable(chunk!),
              (module) => module.identifier().replaceAll('\\', '/'),
            ).join('\n');
            expect(identifiers).toContain(`/${included}`);
            expect(identifiers).not.toContain(`/${excluded}`);
          }
        });
      });
    }),
  ],
});

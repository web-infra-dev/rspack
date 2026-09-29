import { defineConfig } from '@rspack/cli';
import { type Compiler, rspack } from '@rspack/core';

const PLUGIN_NAME = 'plugin';

const SYMBOL = Symbol('mark');

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: PLUGIN_NAME,
          stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
        },
        () => {
          compilation.emitAsset(
            '/foo.txt',
            new compiler.rspack.sources.RawSource('foo'),
            {
              bool: true,
              number: 1,
              string: 'foo',
              array: ['foo', 'bar'],
              object: {
                bool: true,
                number: 1,
                string: 'foo',
                array: ['foo', 'bar'],
              },
              [SYMBOL]: 'foo',
            },
          );
        },
      );

      compilation.hooks.afterProcessAssets.tap(PLUGIN_NAME, () => {
        const info = compilation.getAsset('/foo.txt')?.info;

        expect(info?.bool).toBe(true);
        expect(info?.number).toBe(1);
        expect(info?.string).toBe('foo');
        expect(info?.array).toEqual(['foo', 'bar']);
        expect(info?.object).toEqual({
          bool: true,
          number: 1,
          string: 'foo',
          array: ['foo', 'bar'],
        });
        expect(Reflect.get(info!, SYMBOL)).toBe('foo');
      });
    });
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
});

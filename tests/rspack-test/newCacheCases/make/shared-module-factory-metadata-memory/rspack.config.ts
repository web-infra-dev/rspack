import { rspack } from '@rspack/core';
import { defineConfig, definePlugin } from '@rspack/cli';

const { RawSource } = rspack.sources;
const pluginName = 'SharedModuleCache-memory';

let buildIndex = 0;
let builds = 0;
let succeeded = 0;
let reused = 0;
let completed = 0;

export default defineConfig({
  entry: './d.js',
  cache: { type: 'memory' },
  experiments: {
    newCache: {
      module: true,
      loader: false,
      codeGeneration: false,
      devtool: false,
      minimize: false,
    },
  },
  module: { rules: [{ test: /d\.js$/, sideEffects: false }] },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(pluginName, (compilation) => {
          compilation.hooks.buildModule.tap(pluginName, () => builds++);
          compilation.hooks.succeedModule.tap(pluginName, (module) => {
            succeeded++;
            expect(module.factoryMeta.sideEffectFree).toBe(true);
            module.factoryMeta = { sideEffectFree: false };
            expect(module.factoryMeta.sideEffectFree).toBe(false);
            module.emitFile('built.txt', new RawSource('build output'));
          });
          compilation.hooks.stillValidModule.tap(pluginName, (module) => {
            reused++;
            expect(module.factoryMeta.sideEffectFree).toBe(true);
            expect(
              (module as { resource?: string }).resource?.replace(
                /^.*[\\/]/,
                '',
              ),
            ).toBe('d.js');
          });
          compilation.hooks.seal.tap(pluginName, () => {
            const module = [...compilation.modules].find(
              (module) =>
                (module as { resource?: string }).resource?.replace(
                  /^.*[\\/]/,
                  '',
                ) === 'd.js',
            );
            expect(module!.originalSource()!.source()).toContain(
              'module.exports',
            );
            completed++;
          });
        });
        compiler.hooks.done.tap(pluginName, (stats) => {
          expect(stats.compilation.getAsset('built.txt')!.source.source()).toBe(
            'build output',
          );
          if (buildIndex === 0) {
            // The memory cache does not survive a fresh compiler.
            expect({ builds, succeeded, reused, completed }).toEqual({
              builds: 1,
              succeeded: 1,
              reused: 0,
              completed: 1,
            });
          } else {
            expect({ builds, succeeded, reused, completed }).toEqual({
              builds: 2,
              succeeded: 2,
              reused: 0,
              completed: 2,
            });
          }
          buildIndex++;
        });
      },
    }),
  ],
});

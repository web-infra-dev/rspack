import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
import { experiments } from '@rspack/core';

export default defineConfig({
  mode: 'production',
  target: 'web',
  devtool: false,
  cache: false,
  incremental: false,
  entry: './index.js',
  output: {
    filename: 'main.js',
    chunkFilename: '[name].js',
  },
  optimization: {
    splitChunks: false,
    minimize: false,
    concatenateModules: false,
    usedExports: false,
    moduleIds: 'named',
    chunkIds: 'named',
  },
  plugins: [
    new experiments.VirtualModulesPlugin({
      'index.js': `
        export const load0 = () => import(/* webpackChunkName: 'g0' */ './a0.js');
        export const load1 = () => import(/* webpackChunkName: 'g1' */ './a1.js');
        it('settles availability when a late parent closes a cycle', async () => {
          const direct = await load0();
          expect(direct.values).toEqual([3]);
          expect((await direct.load0()).values).toEqual([3]);
          const a1 = await load1();
          const a2 = await a1.load1();
          const again = await a2.load0();
          expect(typeof again.load1).toBe('function');
          expect(typeof (await again.load1()).load0).toBe('function');
        });
      `,
      'a0.js': `
        import { v } from './m3.js';
        export const values = [v];
        export const load0 = () => import(/* webpackChunkName: 'g2' */ './a3.js');
      `,
      'a1.js': `export const load1 = () => import(/* webpackChunkName: 'g3' */ './a2.js');`,
      'a2.js': `export const load0 = () => import(/* webpackChunkName: 'g2' */ './a1.js');`,
      'a3.js': `import { v } from './m3.js'; export const values = [v];`,
      'm3.js': `
        export const v = 3;
        export const load = () => import(/* webpackChunkName: 'g1' */ './a0.js');
      `,
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'AssertAvailabilityCycle',
          (compilation) => {
            compilation.hooks.afterOptimizeModules.tap(
              'AssertAvailabilityCycle',
              () => {
                const g2 = compilation.namedChunkGroups.get('g2')!;
                const g3 = compilation.namedChunkGroups.get('g3')!;
                assert.ok(
                  g2.getParents().some((parent) => parent.name === 'g3'),
                );
                assert.ok(
                  g3.getParents().some((parent) => parent.name === 'g2'),
                );
              },
            );
          },
        );
      },
    }),
  ],
});

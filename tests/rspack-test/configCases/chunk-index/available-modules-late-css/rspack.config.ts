import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
import { Compilation, experiments } from '@rspack/core';

export default [false, true].map((late, index) =>
  defineConfig({
    mode: 'production',
    target: 'web',
    devtool: false,
    cache: false,
    incremental: false,
    entry: './index.js',
    output: {
      filename: `main-${index}.js`,
      chunkFilename: `[name]-${index}.js`,
      cssChunkFilename: `[name]-${index}.css`,
    },
    module: {
      rules: [{ test: /\.css$/, type: 'css/auto' }],
    },
    optimization: {
      splitChunks: false,
      minimize: false,
      concatenateModules: false,
      usedExports: false,
      sideEffects: false,
      moduleIds: 'named',
      chunkIds: 'named',
    },
    plugins: [
      new experiments.VirtualModulesPlugin({
        'index.js': `
        export const load = () => import(/* webpackChunkName: 'parent' */ './parent-a');
        export const later = () => import(/* webpackChunkName: 'q' */ './q');
      `,
        'parent-a.js': `
        export { v } from './chain-0';
        export const a = () => import(/* webpackChunkName: 'child' */ './child-a');
        export const b = () => import(/* webpackChunkName: 'child' */ './child-b');
      `,
        'q.js': `export const load = () => import(/* webpackChunkName: 'r' */ './r');`,
        'r.js': `export const load = () => import(/* webpackChunkName: 'parent' */ './parent-b');`,
        'parent-b.js': late
          ? `export { x } from './x';`
          : 'export const other = 0;',
        'child-a.js': `export { v } from './chain-0'; export { x } from './x'; import './a.css';`,
        'child-b.js': `import './b.css'; export const b = 1;`,
        'chain-0.js': `export { v } from './chain-1';`,
        'chain-1.js': `export { v } from './chain-2';`,
        'chain-2.js': 'export const v = 42;',
        'x.js': 'export const x = 1;',
        'a.css': '.same { color: red; }',
        'b.css': '.same { color: blue; }',
      }),
      definePlugin({
        apply(compiler) {
          compiler.hooks.compilation.tap(
            'AssertStableCssOrder',
            (compilation) => {
              compilation.hooks.processAssets.tap(
                {
                  name: 'AssertStableCssOrder',
                  stage: Compilation.PROCESS_ASSETS_STAGE_REPORT,
                },
                () => {
                  const css = compilation
                    .getAsset(`child-${index}.css`)!
                    .source.source()
                    .toString();
                  assert.match(css, /color: red;[\s\S]*color: blue;/);
                  const child = compilation.namedChunks.get('child')!;
                  const hasX = [
                    ...compilation.chunkGraph.getChunkModulesIterable(child),
                  ].some((module) => /[\\/]x\.js$/.test(module.identifier()));
                  assert.equal(
                    hasX,
                    !late,
                    'late JS deduplication preserves CSS order',
                  );
                },
              );
            },
          );
        },
      }),
    ],
  }),
);

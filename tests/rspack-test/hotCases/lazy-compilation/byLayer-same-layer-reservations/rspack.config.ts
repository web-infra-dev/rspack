import { defineConfig, definePlugin } from '@rspack/cli';
import { IgnorePlugin } from '@rspack/core';

export default defineConfig(
  [false, true].flatMap((dynamic) =>
    [false, true].map((layered) => {
      const values = {
        './local': 'path',
        ignored: 'missing-reserved-external',
      };
      return {
        output: { library: { name: 'TestLib', type: 'umd' } },
        externalsType: 'umd',
        externals: [
          ({ request }) => (request === './local' ? false : undefined),
          {
            byLayer: dynamic ? () => values : { server: values, null: values },
          },
        ],
        plugins: [
          new IgnorePlugin({ resourceRegExp: /^ignored$/ }),
          definePlugin((compiler) => {
            let builds = 0;
            compiler.hooks.afterCompile.tap(
              'AssertDeferredLocal',
              (compilation) => {
                const localBuilt = [...compilation.modules].some((module) =>
                  module.identifier().includes('local.js'),
                );
                expect(localBuilt).toBe(builds++ > 0);
              },
            );
          }),
        ],
        module: {
          rules: layered ? [{ test: /module\.js$/, layer: 'server' }] : [],
        },
        lazyCompilation: { entries: false },
      };
    }),
  ),
);

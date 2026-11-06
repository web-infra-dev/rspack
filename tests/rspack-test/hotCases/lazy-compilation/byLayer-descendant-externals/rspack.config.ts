import { defineConfig, definePlugin } from '@rspack/cli';
import { IgnorePlugin } from '@rspack/core';

export default defineConfig({
  output: { library: { name: 'TestLib', type: 'umd' } },
  externalsType: 'umd',
  externals: [
    ({ request }) => (request === './local' ? false : undefined),
    {
      byLayer: {
        server: {
          bridge: 'path',
          './local': 'path',
          ignored: 'missing-reserved-external',
        },
        client: { bridge: 'fs', './local': false, ignored: false },
      },
    },
  ],
  plugins: [
    new IgnorePlugin({ resourceRegExp: /^ignored$/ }),
    definePlugin((compiler) => {
      let builds = 0;
      compiler.hooks.afterCompile.tap('AssertDeferredChild', (compilation) => {
        const localBuilt = [...compilation.modules].some((module) =>
          module.identifier().includes('local.js'),
        );
        expect(localBuilt).toBe(builds++ > 0);
      });
    }),
  ],
  module: {
    rules: [
      { test: /server\.js$/, oneOf: [{ layer: 'server' }] },
      { test: /client\.js$/, rules: [{ layer: 'client' }] },
    ],
  },
  lazyCompilation: { entries: false },
});

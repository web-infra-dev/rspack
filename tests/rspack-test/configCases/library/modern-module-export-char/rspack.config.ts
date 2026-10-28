import { defineConfig, definePlugin } from '@rspack/cli';
import { type Compilation } from '@rspack/core';

export default defineConfig({
  entry: {
    index: './index.js',
  },
  output: {
    filename: `[name].js`,
    module: true,
    library: { type: 'modern-module' },
    iife: false,
    chunkFormat: 'module',
  },
  externalsType: 'module-import',
  externals: 'external-module',
  optimization: {
    runtimeChunk: false,
  },
  plugins: [
    definePlugin(function () {
      const handler = (compilation: Compilation) => {
        compilation.hooks.afterProcessAssets.tap('testcase', (assets) => {
          const bundle = Object.values(assets)[0].source().toString();
          if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
            expect(bundle).toContain(
              `var foo_default = /*#__PURE__*/compatGetDefaultExport(foo);\nvar foo_default_0 = foo_default();`,
            );
          } else {
            expect(bundle).toContain(
              `var foo_default = /*#__PURE__*/__webpack_require__.n(foo);\nvar foo_default_0 = foo_default();`,
            );
          }
          expect(bundle).toContain('foo_default_0 as cjsInterop');
          expect(bundle).toContain(
            'export { default as defaultImport, namedImport } from "external-module";',
          );
        });
      };
      this.hooks.compilation.tap('testcase', handler);
    }),
  ],
});

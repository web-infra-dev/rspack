import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [{ test: /index\.js$/, loader: './loader.mjs' }],
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.afterCompile.tap('CheckLoaderStacks', (compilation) => {
        for (const diagnostic of [
          compilation.errors[0],
          compilation.warnings[0],
        ]) {
          assert.match(
            diagnostic.details!,
            /^\s+at .*stackCleanupLoader[^\n]*$/,
          );
        }
        assert.equal(
          compilation.errors[1].details,
          '    at customLoader (/project/LOADER_EXECUTION.js:1:1)\n    at caller (/project/caller.js:2:1)',
        );
        assert.equal(compilation.errors[2].details, '');
        assert.equal(
          compilation.errors[3].details,
          '    at parse (/loader.js:10:5)',
        );
        assert.equal(
          compilation.errors[4].details,
          '    at run (/loader.js:10:5)',
        );
      });
    }),
  ],
});

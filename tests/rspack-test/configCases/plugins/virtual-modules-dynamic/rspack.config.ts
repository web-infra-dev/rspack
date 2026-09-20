import { defineConfig, definePlugin } from '@rspack/cli';

import { rspack } from '@rspack/core';

const {
  experiments: { VirtualModulesPlugin },
} = rspack;

export default defineConfig({
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    definePlugin(function test(compiler) {
      const plugin = new VirtualModulesPlugin({});
      plugin.apply(compiler);

      compiler.hooks.thisCompilation.tap('test', () => {
        plugin.writeModule('foo.js', 'export const foo = "foo"');
        plugin.writeModule('bar.js', 'export const bar = "bar"');
      });
    }),
  ],
});

import { rspack } from '@rspack/core';

const {
  experiments: { VirtualModulesPlugin },
} = rspack;

/** @type {import("@rspack/core").Configuration} */
export default {
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
    function test(compiler) {
      const plugin = new VirtualModulesPlugin({});
      plugin.apply(compiler);

      compiler.hooks.thisCompilation.tap('test', () => {
        plugin.writeModule('foo.js', 'export const foo = "foo"');
        plugin.writeModule('bar.js', 'export const bar = "bar"');
      });
    },
  ],
};

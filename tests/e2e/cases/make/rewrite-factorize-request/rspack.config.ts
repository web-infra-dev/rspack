import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import { rspack } from '@rspack/core';

const sharedObj = {
  time: 1,
};

export default defineConfig({
  entry: './index.js',
  context: import.meta.dirname,
  cache: true,
  incremental: true,
  plugins: [
    new rspack.HtmlRspackPlugin(),
    {
      apply(compiler: Compiler) {
        Object.assign(compiler, { __sharedObj: sharedObj });
        compiler.hooks.compilation.tap(
          'PLUGIN',
          (_, { normalModuleFactory }) => {
            normalModuleFactory.hooks.resolve.tapPromise(
              'PLUGIN',
              async (resolveData) => {
                if (resolveData.request == './file.js') {
                  resolveData.request = `./loader.mjs?time=${sharedObj.time}!./file.js`;
                }
              },
            );
          },
        );
      },
    },
  ],
});

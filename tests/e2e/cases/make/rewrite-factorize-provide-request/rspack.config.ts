import type { Configuration, Compiler } from '@rspack/core';
import { rspack } from '@rspack/core';

const sharedObj = {
  time: 1,
};

export default {
  entry: './index.js',
  context: import.meta.dirname,
  cache: true,
  incremental: true,
  plugins: [
    new rspack.HtmlRspackPlugin(),
    new rspack.ProvidePlugin({
      providedDefault: ['./provided.js', 'default'],
      providedNamed: ['./provided.js', 'named'],
    }),
    {
      apply(compiler: Compiler) {
        Object.assign(compiler, { __sharedObj: sharedObj });
        compiler.hooks.compilation.tap(
          'PLUGIN',
          (_, { normalModuleFactory }) => {
            normalModuleFactory.hooks.resolve.tapPromise(
              'PLUGIN',
              async (resolveData) => {
                if (resolveData.request == './provided.js') {
                  resolveData.request = `./loader.mjs?time=${sharedObj.time}!./provided.js`;
                }
              },
            );
          },
        );
      },
    },
  ],
} satisfies Configuration;

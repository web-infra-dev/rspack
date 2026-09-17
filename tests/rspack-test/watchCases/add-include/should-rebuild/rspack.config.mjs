import { rspack } from '@rspack/core';
import path from 'node:path';

/**@type {import('@rspack/core').Configuration} */
const config = {
  entry: {
    main: './index.js',
  },
  mode: 'development',
  plugins: [
    {
      apply(
        /**@type {import('@rspack/core').Compiler} */
        compiler,
      ) {
        let initial = true;

        compiler.hooks.finishMake.tapPromise('test', async (compilation) => {
          if (initial) {
            initial = false;
            return Promise.resolve();
          }

          return new Promise((resolve, reject) => {
            const dependency = rspack.EntryPlugin.createDependency(
              path.resolve(import.meta.dirname, './plugin-included.js'),
            );

            compilation.addInclude(
              compiler.context,
              dependency,
              { name: 'main' },
              (err) => {
                if (err) {
                  reject(new Error(`Error adding entry: ${err}`));
                } else {
                  resolve();
                }
              },
            );
          });
        });
      },
    },
  ],
  incremental: true,
};

export default config;

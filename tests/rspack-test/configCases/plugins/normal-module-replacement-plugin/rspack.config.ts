import { defineConfig } from '@rspack/cli';
import { NormalModuleReplacementPlugin } from '@rspack/core';
import path from 'node:path';

export default /** @type {import("@rspack/core").Configuration} */ defineConfig(
  {
    plugins: [
      new NormalModuleReplacementPlugin(/request.v1(\.|$)/, (r) => {
        r.request = r.request.replace(/request\.v1(\.|$)/, 'request.v2$1');
      }),
      new NormalModuleReplacementPlugin(
        /resource\.foo\.js$/,
        ({ createData }) => {
          if (createData && createData.resource) {
            createData.resource = createData.resource.replace(
              /resource\.foo\.js$/,
              'resource.bar.js',
            );
          }
        },
      ),
      new NormalModuleReplacementPlugin(
        /[/\\]query[/\\]query.v1(\.|$)/,
        path.resolve(import.meta.dirname, './query/query.v2.js'),
      ),
    ],
  },
);

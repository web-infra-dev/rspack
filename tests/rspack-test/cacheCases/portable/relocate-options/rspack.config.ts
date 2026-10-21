import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: (info) => {
          if (info.resource === undefined) return [];
          return [
            {
              loader: './loader.cjs',
              options: {
                input: path.join(path.dirname(info.resource), 'value.txt'),
              },
            },
          ];
        },
      },
    ],
  },
});

import { defineConfig } from '@rspack/cli';

export default /** @type {import("@rspack/core").Configuration} */ defineConfig(
  {
    entry: './index.ts',
    resolve: {
      extensions: ['...', '.ts'],
    },
    module: {
      parser: {
        javascript: {
          typeReexportsPresence: 'tolerant',
        },
      },
      rules: [
        {
          test: /\.ts$/,
          use: [
            {
              loader: 'builtin:swc-loader',
              options: {
                detectSyntax: 'auto',
                collectTypeScriptInfo: {
                  typeExports: true,
                },
              },
            },
          ],
        },
      ],
    },
  },
);

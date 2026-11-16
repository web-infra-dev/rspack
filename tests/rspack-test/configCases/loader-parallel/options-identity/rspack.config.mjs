import { DefinePlugin } from '@rspack/core';

export default [true, 'strict'].flatMap((parallel) =>
  ['development', 'production'].map((mode) => ({
    mode,
    cache: false,
    incremental: false,
    optimization: { minimize: false },
    plugins: [new DefinePlugin({ EXPECTED_MODE: JSON.stringify(mode) })],
    module: {
      rules: [
        { test: /values\.js$/, loader: './entry-loader.js' },
        {
          test: /value\.js$/,
          use: [
            {
              loader: './identity-loader.js',
              parallel,
              ident: 'shared-identity',
              options: { policy: String(parallel), nested: { value: 42 } },
            },
          ],
        },
      ],
    },
  })),
);

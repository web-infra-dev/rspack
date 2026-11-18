import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].map((dynamic) => {
    const entry = {
      main: './index.js',
      document: 'document-alias',
    };

    return {
      target: 'node',
      entry: dynamic ? () => entry : entry,
      output: {
        filename: '[name].js',
      },
      resolve: {
        alias: {
          'document-alias': ['./custom.js', './fallback.js'],
        },
      },
    };
  }),
);

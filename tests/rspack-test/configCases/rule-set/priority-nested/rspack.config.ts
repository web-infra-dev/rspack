import { defineConfig } from '@rspack/cli';

export default defineConfig(
  ['sync', 'async', 'fallback'].map((mode) => {
    const condition = (pattern: RegExp) =>
      mode === 'async' ? (value: string) => pattern.test(value) : pattern;
    return {
      module: {
        rules: [
          {
            test: condition(/\.txt$/),
            resourceQuery: condition(/child/),
            loader: './reject-loader.mjs',
            rules: [
              { priority: 2, type: 'asset/source' },
              { priority: 1, loader: './reject-loader.mjs' },
            ],
          },
          {
            test: condition(/\.txt$/),
            resourceQuery: condition(/parent/),
            priority: 3,
            type: 'asset/source',
            rules: [{ loader: './reject-loader.mjs' }],
          },
          {
            test: condition(/\.txt$/),
            resourceQuery: condition(/^\?one-of$/),
            oneOf: [
              { resourceQuery: condition(/never-matches/), priority: 100 },
              { priority: 2, type: 'asset/source' },
              { priority: 10, loader: './reject-loader.mjs' },
            ],
          },
          {
            test: condition(/\.txt$/),
            resourceQuery: condition(/one-of-parent/),
            priority: 3,
            type: 'asset/source',
            oneOf: [{ loader: './reject-loader.mjs' }],
          },
          {
            test: condition(/\.txt$/),
            resourceQuery: condition(/suppressed/),
            priority: 4,
            type: 'asset/source',
          },
          {
            test: condition(/\.txt$/),
            resourceQuery: condition(/suppressed/),
            oneOf: [
              { priority: 1, loader: './reject-loader.mjs' },
              { priority: 10, loader: './reject-loader.mjs' },
            ],
          },
          {
            test: condition(/\.txt$/),
            resourceQuery: condition(/cross-level/),
            priority: 1,
            loader: './reject-loader.mjs',
          },
          {
            test: condition(/\.txt$/),
            resourceQuery: condition(/cross-level/),
            rules: [
              {
                priority: 1,
                loader: './reject-loader.mjs',
                rules: [{ priority: 2, type: 'asset/source' }],
              },
            ],
          },
          {
            test: condition(/\.txt$/),
            resourceQuery: condition(/cross-level/),
            priority: 1,
            loader: './reject-loader.mjs',
          },
          {
            test: condition(/never-matches$/),
            rules: [{ priority: 100, loader: './reject-loader.mjs' }],
          },
          // Fall back after synchronous matching has already replaced effects.
          ...(mode === 'fallback'
            ? [
                {
                  test: () => false,
                  priority: 100,
                  loader: './reject-loader.mjs',
                },
              ]
            : []),
        ],
      },
    };
  }),
);

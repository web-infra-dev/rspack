const path = require('path');

const loader = (name) => path.resolve(__dirname, `${name}-loader.js`);
const owner = (kind) => ({ loader: loader('owner'), options: { kind } });
const left = { loader: loader('left'), cache: true };

module.exports = {
  mode: 'development',
  cache: { type: 'memory' },
  incremental: false,
  experiments: {
    newCache: { loader: true, codeGeneration: false, minimize: false },
  },
  module: {
    rules: [
      ...['file', 'build'].flatMap((kind) =>
        [false, true].map((parallel) => ({
          resourceQuery: new RegExp(`^\\?${kind}-${parallel}$`),
          use: [
            left,
            {
              loader: loader('value'),
              options: { kind },
              cache: true,
              parallel: parallel ? { maxWorkers: 1 } : false,
            },
            owner(kind),
          ],
        })),
      ),
      {
        resourceQuery: /^\?native$/,
        use: [
          left,
          { loader: 'builtin:test-dependency-loader', cache: true },
          owner('file'),
        ],
      },
      {
        resourceQuery: /^\?mixed$/,
        use: [
          left,
          { loader: 'builtin:test-no-passthrough-loader', cache: true },
          { loader: loader('value'), options: { kind: 'file' }, cache: true },
          owner('file'),
        ],
      },
      {
        resourceQuery: /^\?removed$/,
        use: [
          left,
          { loader: loader('value'), options: { kind: 'file' }, cache: true },
          { ...owner('file'), cache: true },
          { loader: loader('clear'), cache: true },
          owner('file'),
        ],
      },
    ],
  },
};

const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

module.exports = [false, true].flatMap((rslib) =>
  [false, true].map((minimize) => ({
    name: `${rslib ? 'rslib' : 'rspack'}-${minimize ? 'min' : 'plain'}`,
    entry: {
      named: './named.js',
      star: './star.js',
      namespace: './namespace.js',
      default: './default.js',
      reverse: './reverse.js',
      mixed: './mixed.js',
      duplicate: './duplicate.js',
      wrapped: './wrapped.js',
      descendant: './descendant.js',
      descendantBefore: './descendant-before.js',
    },
    output: {
      filename: `${rslib ? 'rslib' : 'rspack'}-${minimize ? 'min' : 'plain'}/[name].mjs`,
    },
    externals: {
      'order-a': 'module ../fixtures/a.mjs',
      'order-b': 'module ../fixtures/b.mjs',
      'order-c': 'module ../fixtures/c.mjs',
    },
    optimization: { minimize, runtimeChunk: false },
    plugins: rslib ? [new RslibPlugin()] : [],
  })),
);

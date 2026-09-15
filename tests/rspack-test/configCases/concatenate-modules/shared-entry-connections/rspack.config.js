const assert = require('node:assert/strict');
const path = require('node:path');

/** @type {import('@rspack/core').Configuration[]} */
module.exports = ['webpack', 'rspack'].flatMap((runtimeMode) =>
  [false, true].map((concatenateSharedEntry) => ({
    mode: 'production',
    entry: {
      entry1: ['./index.js', './bar.js'],
      entry2: ['./index-alt.js', './bar.js'],
    },
    output: {
      filename: `${runtimeMode}-${concatenateSharedEntry}-[name].js`,
    },
    experiments: { runtimeMode },
    resolve: {
      alias: {
        './bar.js': path.resolve(
          __dirname,
          concatenateSharedEntry ? 'bar-concatenated.js' : 'bar.js',
        ),
      },
    },
    optimization: {
      concatenateModules: true,
      // Keep value.js available for the shared entry's concatenation group.
      inlineExports: !concatenateSharedEntry,
      minimize: false,
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.afterCompile.tap(
            'CheckCopiedConnections',
            (compilation) => {
              const graph = compilation.moduleGraph;
              const modules = [...compilation.modules];
              const foo = modules.find(
                (m) => m.resource === path.join(__dirname, 'foo.js'),
              );
              assert.ok(foo);
              const groups = modules.filter(
                (m) => m.modules && [...m.modules].includes(foo),
              );
              assert.equal(groups.length, 2);
              for (const original of graph.getOutgoingConnections(foo)) {
                const copies = [
                  ...graph.getIncomingConnections(original.module),
                ].filter((c) => c.dependency === original.dependency);
                assert.equal(copies.length, 3);
                assert.equal(new Set(copies).size, 3);
                assert.deepEqual(
                  new Set(copies.map((c) => c.originModule)),
                  new Set([foo, ...groups]),
                );
                assert.equal(
                  graph.getConnection(original.dependency),
                  original,
                );
                for (const copy of copies) {
                  assert.ok(
                    [
                      ...graph.getOutgoingConnections(copy.originModule),
                    ].includes(copy),
                  );
                  assert.equal(copy.dependency._parentModule, foo);
                }
              }
            },
          );
        },
      },
    ],
  })),
);

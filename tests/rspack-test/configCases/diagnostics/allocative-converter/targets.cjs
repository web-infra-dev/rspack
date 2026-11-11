const assert = require('node:assert/strict');
const path = require('node:path');
const { rspack } = require('@rspack/core');

// Keep both owners and old public Compilations alive across captures. A snapshot
// of one compiler must not attach another compiler's JS objects to its Rust tree.
const compilations = [];
const compilers = ['first', 'second'].map((name) =>
  rspack({
    name,
    mode: 'development',
    context: process.env.ALLOCATIVE_TEST_CONTEXT,
    entry: './index.js',
    output: { path: path.join(process.env.ALLOCATIVE_TEST_CONTEXT, name) },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.thisCompilation.tap(
            'RetainCompilations',
            (compilation) => {
              compilations.push(compilation);
            },
          );
        },
      },
    ],
  }),
);

const run = (compiler) =>
  new Promise((resolve, reject) => {
    compiler.run((error, stats) => {
      if (error) return reject(error);
      assert.equal(stats.hasErrors(), false, stats.toString());
      resolve();
    });
  });

(async () => {
  try {
    await run(compilers[0]);
    await run(compilers[1]);
    await run(compilers[0]); // Rebuild replaces Compilation in the same native slot.
    assert.equal(compilations.length, 3);
  } finally {
    await Promise.all(
      compilers.map(
        (compiler) =>
          new Promise((resolve, reject) => {
            compiler.close((error) => (error ? reject(error) : resolve()));
          }),
      ),
    );
  }
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

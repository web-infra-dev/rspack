const path = require('path');

function findForeignSeparator(paths) {
  const foreign = path.sep === '\\' ? '/' : '\\';
  return [...paths].filter((p) => p.includes(foreign));
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  cache: {
    type: 'persistent',
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.afterEmit.tap('AssertNativeSeparator', (compilation) => {
          for (const [kind, paths] of [
            ['file', compilation.fileDependencies],
            ['context', compilation.contextDependencies],
            ['missing', compilation.missingDependencies],
            ['build', compilation.buildDependencies],
          ]) {
            const foreign = findForeignSeparator(paths);
            if (foreign.length) {
              compilation.errors.push(
                new Error(
                  `${kind} dependencies are not spelled with the native separator: ${foreign.join(', ')}`,
                ),
              );
            }
          }
        });
      },
    },
  ],
};

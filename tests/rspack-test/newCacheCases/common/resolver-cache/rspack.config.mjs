import path from 'node:path';

let buildIndex = 0;

/** @type {import('@rspack/core').Configuration} */
export default {
  mode: 'development',
  incremental: false,
  experiments: {
    newCache: {
      resolver: true,
      module: false,
      codeGeneration: false,
      devtool: false,
      loader: false,
      minimize: false,
    },
  },
  cache: {
    type: 'persistent',
    snapshot: {
      resolve: { timestamp: false, hash: true },
    },
  },
  resolve: {
    extensions: ['.json', '.js'],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('ResolverCacheTest', ({ compilation }) => {
          const preferred = buildIndex === 2;
          expect(
            compilation.fileDependencies.has(
              path.join(
                compiler.context,
                preferred ? 'value.json' : 'value.js',
              ),
            ),
          ).toBe(true);
          expect(
            compilation.fileDependencies.has(
              path.join(compiler.context, 'package/package.json'),
            ),
          ).toBe(true);
          if (!preferred) {
            expect(
              compilation.missingDependencies.has(
                path.join(compiler.context, 'value.json'),
              ),
            ).toBe(true);
          }
          buildIndex++;
        });
      },
    },
  ],
};

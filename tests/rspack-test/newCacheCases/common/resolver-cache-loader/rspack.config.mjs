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
  resolveLoader: {
    extensions: ['.cjs', '.js'],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap(
          'LoaderResolverCacheTest',
          ({ compilation }) => {
            const preferred = buildIndex === 2;
            expect(
              compilation.fileDependencies.has(
                path.join(
                  compiler.context,
                  preferred ? 'convert.cjs' : 'convert.js',
                ),
              ),
            ).toBe(true);
            if (!preferred) {
              expect(
                compilation.missingDependencies.has(
                  path.join(compiler.context, 'convert.cjs'),
                ),
              ).toBe(true);
            }
            buildIndex++;
          },
        );
      },
    },
  ],
};

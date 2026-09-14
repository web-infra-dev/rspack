const path = require('node:path');

const TOTAL_BUILDS = 3;

let build = 0;
const seen = [];

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /dep\.js$/,
        use: [{ loader: require.resolve('./noop-loader.cjs') }],
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('PLUGIN', (compilation) => {
          build++;
          const currentBuild = build;

          compiler.webpack.NormalModule.getCompilationHooks(
            compilation,
          ).beforeLoaders.tap('PLUGIN', (loaders, mod) => {
            if (!/dep\.js$/.test(mod.resource || '')) return;
            seen.push({
              build: currentBuild,
              incoming: loaders.map((l) => path.basename(String(l.loader))),
            });
            // Appended unconditionally on every build. The list a tap receives is
            // derived from `module.rules` each time a module is built, so this must
            // not accumulate across rebuilds even though `loaders` is part of the
            // module's cacheable state.
            loaders.push({ loader: require.resolve('./tag-loader.cjs') });
          });

          compilation.hooks.seal.tap('PLUGIN', () => {
            if (currentBuild < TOTAL_BUILDS) return;
            // Asserted on the last build so that a tap which stopped running would
            // fail here rather than let the case pass without checking anything.
            expect(seen).toEqual([
              { build: 1, incoming: ['noop-loader.cjs'] },
              { build: 2, incoming: ['noop-loader.cjs'] },
              { build: 3, incoming: ['noop-loader.cjs'] },
            ]);
          });
        });
      },
    },
  ],
};

const { CopyRspackPlugin } = require('@rspack/core');
const path = require('path');

module.exports = {
  entry: './index.js',
  target: 'node',
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          // A glob `from` registers its stable, non-magic base directory as a
          // context dependency so new sibling matches are observed. The base must
          // be normalized to stay consistent with the rest of the dependency graph.
          from: './public/**/*',
        },
      ],
    }),
    {
      apply(compiler) {
        compiler.hooks.done.tap('DonePlugin', (stats) => {
          expect([...stats.compilation.contextDependencies]).toContain(
            path.resolve(__dirname, 'public'),
          );
          for (const dir of stats.compilation.contextDependencies) {
            expect(dir).toBe(path.normalize(dir));
          }
        });
      },
    },
  ],
};

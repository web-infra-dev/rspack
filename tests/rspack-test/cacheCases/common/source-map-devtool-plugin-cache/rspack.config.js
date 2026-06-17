const PLUGIN_NAME = 'rspack.SourceMapDevToolPlugin';

let updateIndex = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  devtool: 'source-map',
  output: {
    filename: 'bundle.js',
    chunkFilename: '[name].chunk.js',
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('SourceMapPersistentCacheTest', (stats) => {
          const s = stats.toJson({
            all: false,
            logging: 'verbose',
          });

          const logEntries = s.logging[PLUGIN_NAME]?.entries ?? [];
          const cacheLogEntry = logEntries.find(
            (e) =>
              e.type === 'cache' &&
              e.message &&
              e.message.startsWith('source map persistent cache:'),
          );
          expect(cacheLogEntry).toBeTruthy();

          const match = cacheLogEntry.message.match(
            /source map persistent cache: [\d.]+% \((\d+)\/(\d+)\)/,
          );
          expect(match).toBeTruthy();

          const hits = parseInt(match[1], 10);
          const total = parseInt(match[2], 10);
          const misses = total - hits;

          if (updateIndex === 0) {
            expect(hits).toBe(0);
            expect(misses).toBe(2);
          }
          if (updateIndex === 1) {
            expect(hits).toBe(2);
            expect(misses).toBe(0);
          }
          if (updateIndex === 2) {
            expect(hits).toBe(1);
            expect(misses).toBe(1);
          }

          updateIndex++;
        });
      },
    },
  ],
};

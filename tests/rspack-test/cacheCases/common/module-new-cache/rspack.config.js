let compilerIndex = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  experiments: {
    newCache: true,
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('Test Plugin', (stats) => {
          const logging = stats.toJson({
            all: false,
            logging: 'verbose',
          }).logging;
          const entries = logging['rspack.Compilation']?.entries ?? [];
          const cacheEntry = entries.find(
            (entry) =>
              entry.type === 'cache' &&
              entry.message?.startsWith('module build cache:'),
          );
          expect(cacheEntry).toBeTruthy();

          const match = cacheEntry.message.match(/\((\d+)\/(\d+)\)/);
          expect(match).toBeTruthy();
          const hits = Number(match[1]);
          const total = Number(match[2]);
          expect(total).toBeGreaterThan(0);

          if (compilerIndex === 0) {
            expect(hits).toBe(0);
          } else {
            expect(hits).toBe(total);
          }
          compilerIndex++;
        });
      },
    },
  ],
};

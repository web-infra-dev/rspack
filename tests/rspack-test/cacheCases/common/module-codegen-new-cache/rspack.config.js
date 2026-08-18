let updateIndex = 0;

const LOGGER_NAME = 'rspack.Compilation';

function getCodegenCacheStats(logging) {
  const entry = (logging?.[LOGGER_NAME]?.entries ?? []).find(
    (item) =>
      item.type === 'cache' &&
      item.message?.startsWith('module code generation cache:'),
  );
  expect(entry).toBeTruthy();

  const match = entry.message.match(
    /module code generation cache: [\d.]+% \((\d+)\/(\d+)\)/,
  );
  expect(match).toBeTruthy();

  const hits = Number(match[1]);
  const total = Number(match[2]);
  return { hits, misses: total - hits, total };
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  mode: 'development',
  entry: {
    main: './index.js',
    secondary: './secondary.js',
  },
  output: {
    filename: '[name].js',
  },
  cache: {
    type: 'persistent',
  },
  experiments: {
    newCache: true,
  },
  optimization: {
    concatenateModules: false,
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('ModuleCodegenNewCacheTest', (stats) => {
          const result = getCodegenCacheStats(
            stats.toJson({
              all: false,
              logging: 'verbose',
            }).logging,
          );

          if (updateIndex === 0) {
            expect(result.hits).toBe(0);
            expect(result.misses).toBeGreaterThan(0);
          }
          if (updateIndex === 1) {
            expect(result.hits).toBe(result.total);
            expect(result.misses).toBe(0);
          }
          if (updateIndex === 2) {
            // The incremental code generation pass only schedules the changed
            // module. Unchanged modules reuse their recovered artifacts and do
            // not need to query the code generation cache.
            expect(result.hits).toBe(0);
            expect(result.misses).toBe(1);
            expect(result.total).toBe(1);
          }

          updateIndex++;
        });
      },
    },
  ],
};

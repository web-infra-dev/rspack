const PLUGIN_NAME = 'rspack.incremental.modulesHashes';

let updateIndex = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  mode: 'development',
  cache: {
    type: 'persistent',
  },
  optimization: {
    concatenateModules: false,
    inlineExports: false,
    mangleExports: false,
    usedExports: false,
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('ModuleHashesPersistentCacheTest', (stats) => {
          const s = stats.toJson({
            all: false,
            logging: 'verbose',
            loggingDebug: /^rspack\.incremental\.modulesHashes$/,
          });

          const logEntries = s.logging[PLUGIN_NAME]?.entries ?? [];
          const affectedModulesLogEntry = logEntries.find(
            (e) =>
              e.type === 'log' &&
              e.message &&
              e.message.includes('modules are affected'),
          );

          if (updateIndex === 0) {
            expect(affectedModulesLogEntry).toBeUndefined();
          }
          if (updateIndex === 1) {
            expect(affectedModulesLogEntry).toBeTruthy();

            const match = affectedModulesLogEntry.message.match(
              /(\d+) modules are affected, (\d+) in total/,
            );
            expect(match).toBeTruthy();

            const affectedModules = parseInt(match[1], 10);
            const totalModules = parseInt(match[2], 10);

            expect(affectedModules).toBe(2);
            expect(totalModules).toBe(2);
          }

          updateIndex++;
        });
      },
    },
  ],
};

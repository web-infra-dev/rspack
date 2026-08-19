let updateIndex = 0;

const MODULES_HASHES_LOGGER_NAME = 'rspack.incremental.modulesHashes';
const MODULES_CODEGEN_LOGGER_NAME = 'rspack.incremental.modulesCodegen';

function getAffectedLogEntry(logging, loggerName) {
  return (logging?.[loggerName]?.entries ?? []).find(
    (e) =>
      e.type === 'log' &&
      typeof e.message === 'string' &&
      e.message.includes('modules are affected'),
  );
}

function parseAffectedLogEntry(entry) {
  const match = entry?.message.match(
    /(\d+) modules are affected, (\d+) in total/,
  );
  expect(match).toBeTruthy();
  return {
    affected: Number(match[1]),
    total: Number(match[2]),
  };
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  mode: 'development',
  cache: {
    type: 'persistent',
  },
  incremental: {
    buildModuleGraph: true,
    modulesHashes: true,
    modulesCodegen: true,
  },
  optimization: {
    concatenateModules: false,
    innerGraph: false,
    mangleExports: false,
  },
  output: {
    filename: '[name].mjs',
    chunkFilename: '[name].chunk.[fullhash].mjs',
    module: true,
    library: {
      type: 'modern-module',
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap(
          'ModernModuleCodegenPersistentCacheTest',
          (stats) => {
            const s = stats.toJson({
              all: false,
              logging: 'verbose',
            });
            const modulesCodegenAffectedLogEntry = getAffectedLogEntry(
              s.logging,
              MODULES_CODEGEN_LOGGER_NAME,
            );

            if (updateIndex === 0) {
              expect(modulesCodegenAffectedLogEntry).toBeUndefined();
            }

            if (updateIndex === 1) {
              expect(modulesCodegenAffectedLogEntry).toBeTruthy();
              const modulesCodegen = parseAffectedLogEntry(
                modulesCodegenAffectedLogEntry,
              );
              expect(modulesCodegen.affected).toBe(0);
              expect(modulesCodegen.total).toBeGreaterThan(0);
            }

            if (updateIndex === 2) {
              expect(modulesCodegenAffectedLogEntry).toBeTruthy();
              const modulesHashesAffectedLogEntry = getAffectedLogEntry(
                s.logging,
                MODULES_HASHES_LOGGER_NAME,
              );
              expect(modulesHashesAffectedLogEntry).toBeTruthy();
              const modulesCodegen = parseAffectedLogEntry(
                modulesCodegenAffectedLogEntry,
              );
              expect(modulesCodegen.affected).toBeGreaterThan(0);
              expect(modulesCodegen.affected).toBeGreaterThan(2);
              expect(modulesCodegen.total).toBeGreaterThan(0);
              if (modulesCodegen.affected >= modulesCodegen.total) {
                throw new Error(
                  [
                    `Expected module codegen affected modules to be less than total modules.`,
                    `modulesHashes: ${modulesHashesAffectedLogEntry?.message}`,
                    `modulesCodegen: ${modulesCodegenAffectedLogEntry.message}`,
                  ].join('\n'),
                );
              }
            }

            updateIndex++;
          },
        );
      },
    },
  ],
};

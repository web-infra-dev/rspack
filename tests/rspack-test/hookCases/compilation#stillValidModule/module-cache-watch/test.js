let built = 0;
let reused = 0;

/** @type {import('@rspack/test-tools').THookCaseConfig} */
module.exports = {
  description: 'reuses memory cache during watch with incremental compilation disabled',
  snapshotFileFilter: () => false,
  options() {
    return {
      context: __dirname,
      mode: 'development',
      entry: './index.js',
      cache: { type: 'memory' },
      incremental: false,
      experiments: { newCache: { module: true } },
      plugins: [compiler => {
        compiler.hooks.compilation.tap('WatchModuleCache', compilation => {
          compilation.hooks.buildModule.tap('WatchModuleCache', () => built++);
          compilation.hooks.stillValidModule.tap('WatchModuleCache', () => reused++);
        });
      }],
    };
  },
  async compiler(context, compiler) {
    await new Promise((resolve, reject) => {
      let round = 0;
      const watcher = compiler.watch({}, (error, stats) => {
        try {
          if (error) throw error;
          expect(stats.hasErrors()).toBe(false);
          expect(built).toBe(1);
          expect(reused).toBe(round);
          if (round++ === 0) {
            watcher.invalidate();
          } else {
            watcher.close(error => error ? reject(error) : resolve());
          }
        } catch (error) {
          watcher.close(() => reject(error));
        }
      });
    });
  },
};

let entrypoints;

/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
  description:
    'should generate entrypoints from stale stats without accessing live chunks',
  options(context) {
    return {
      context: context.getSource(),
      entry: {
        main: './fixtures/a',
      },
    };
  },
  compiler(_context, compiler) {
    compiler.hooks.done.tap('EntrypointsWithStaleStatsTest', (stats) => {
      Object.defineProperty(stats.compilation, 'chunks', {
        configurable: true,
        get() {
          throw new Error('stats must not access live compilation chunks');
        },
      });
      try {
        entrypoints = stats.toJson({
          all: false,
          entrypoints: true,
        }).entrypoints;
      } finally {
        delete stats.compilation.chunks;
      }
    });
  },
  check() {
    expect(entrypoints).toHaveProperty('main');
    expect(entrypoints.main.assets).toHaveLength(1);
  },
};

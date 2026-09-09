const { config } = require('@rspack/core');
const normalize = (options) => {
  const normalized = config.getNormalizedWebpackOptions(options);
  config.applyWebpackOptionsDefaults(normalized);
  return normalized;
};
module.exports = {
  plugins: [
    {
      apply() {
        for (const mode of ['development', 'production', 'none']) {
          const options = normalize({
            mode,
            cache: true,
            experiments: { newCache: true },
          });
          expect(options.snapshot).toEqual({
            immutablePaths: [],
            unmanagedPaths: [],
            managedPaths: [/^(.+?[\\/]node_modules[\\/])/],
            module:
              mode === 'production'
                ? { hash: true, timestamp: true }
                : { timestamp: true },
            contextModule: { timestamp: true },
            resolve:
              mode === 'production'
                ? { hash: true, timestamp: true }
                : { timestamp: true },
            buildDependencies: { hash: true, timestamp: true },
            resolveBuildDependencies: { hash: true, timestamp: true },
          });
          expect(options.cache.snapshot).toBe(options.snapshot);
          for (const futureDefaults of [false, true]) {
            expect(
              normalize({
                mode,
                cache: true,
                experiments: { newCache: true, futureDefaults },
              }).snapshot,
            ).toEqual(options.snapshot);
          }
        }
        const strategy = Object.freeze({ hash: true });
        const paths = Object.freeze(['/custom']);
        const input = {
          mode: 'production',
          experiments: { newCache: true },
          cache: {
            type: 'memory',
            snapshot: { module: { timestamp: true }, unmanagedPaths: paths },
          },
          snapshot: { module: strategy, resolve: {}, managedPaths: [] },
        };
        const options = normalize(input);
        expect(options.snapshot.module).toEqual({ hash: true });
        expect(options.snapshot.resolve).toEqual({});
        expect(options.snapshot.unmanagedPaths).toEqual(paths);
        expect(options.snapshot.module).not.toBe(strategy);
        expect(options.snapshot.unmanagedPaths).not.toBe(paths);
        expect(options.cache.snapshot).toBe(options.snapshot);
        expect(
          normalize({
            mode: 'production',
            cache: false,
            snapshot: { module: strategy },
          }).snapshot.module,
        ).toEqual(strategy);
      },
    },
  ],
};

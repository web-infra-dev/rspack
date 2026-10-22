import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig(
  (['production', 'development'] as const).flatMap((mode) =>
    [false, true].map((async) => {
      const name = `${mode}-${async ? 'async' : 'initial'}`;
      return defineConfig({
        name,
        mode,
        entry: async ? './index.js' : './modules.js',
        output: {
          filename: `${name}-[name].js`,
          chunkFilename: `${name}-[name].js`,
        },
        optimization: {
          minimize: false,
          concatenateModules: false,
          splitChunks: {
            chunks: 'all',
            minSize: 0,
            enforceSizeThreshold: Infinity,
            cacheGroups: {
              default: false,
              defaultVendors: false,
              shared: {
                test: /[\\/]shared\.js$/,
                name: (module) =>
                  `shared-${module.identifier().split('?').pop()}`,
              },
            },
          },
        },
        plugins: [
          definePlugin(function () {
            this.hooks.compilation.tap(name, (compilation) => {
              compilation.hooks.afterSeal.tap(name, () => {
                const groups = Array.from(compilation.chunkGroups).filter(
                  (group) => group.isInitial() !== async,
                );
                expect(groups).toHaveLength(1);
                expect(groups[0].chunks).toHaveLength(
                  mode === 'production' ? 30 : 32,
                );
              });
            });
          }),
        ],
      });
    }),
  ),
);

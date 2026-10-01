import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RsdoctorPlugin },
} = rspack;

export default defineConfig({
  devtool: false,
  plugins: [
    new RsdoctorPlugin({
      moduleGraphFeatures: false,
      chunkGraphFeatures: false,
      sourceMapFeatures: {
        cheap: true,
        module: true,
      },
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.afterEmit.tap('TestPlugin::SourceMap', (compilation) => {
          const assets = compilation.getAssets();

          // Check if each JS and CSS asset has a source map
          const jsCssAssets = assets.filter(
            (asset) =>
              asset.name.endsWith('.js') || asset.name.endsWith('.css'),
          );

          jsCssAssets.forEach((asset) => {
            expect(asset.source.map()).toBeTruthy();
          });

          const sourceMapAssets = assets.filter((asset) =>
            asset.name.endsWith('.map'),
          );
          expect(sourceMapAssets.length).toBe(0);
        });
      },
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.afterEmit.tap('TestPlugin::CheapOnly', (compilation) => {
          const assets = compilation.getAssets();
          const jsCssAssets = assets.filter(
            (asset) =>
              asset.name.endsWith('.js') || asset.name.endsWith('.css'),
          );
          // Check if each asset has a source map
          jsCssAssets.forEach((asset) => {
            expect(asset.source.map()).toBeTruthy();
          });

          const sourceMapAssets = assets.filter((asset) =>
            asset.name.endsWith('.map'),
          );
          expect(sourceMapAssets.length).toBe(0);
        });
      },
    }),
  ],
});

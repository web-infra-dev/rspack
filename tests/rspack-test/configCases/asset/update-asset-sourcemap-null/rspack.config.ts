import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('PLUGIN', (compilation) => {
          compilation.hooks.processAssets.tap('PLUGIN', (assets) => {
            for (const name in assets) {
              const source = assets[name];
              compilation.updateAsset(name, source, {
                related: { sourceMap: null },
              });
            }
          });
        });
      },
    }),
  ],
});

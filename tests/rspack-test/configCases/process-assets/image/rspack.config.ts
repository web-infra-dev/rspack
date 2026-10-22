import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
      },
    ],
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.compilation.tap('PLUGIN', (compilation) => {
        compilation.hooks.processAssets.tap('PLUGIN', (assets) => {
          for (const name in assets) {
            if (name.endsWith('png')) {
              expect(assets[name].source()).toBeInstanceOf(Buffer);
            }
          }
        });
      });
    }),
  ],
});

import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

let index = 0;
let provideMap = {
  Mod: ['./a', 'default'],
};

export default defineConfig({
  context: import.meta.dirname,
  entry: './entry.js',
  optimization: {
    minimize: false,
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    definePlugin({
      apply(_compiler) {
        if (index === 1) {
          provideMap.Mod[0] = './b';
        }
        index++;
      },
    }),
    new rspack.ProvidePlugin(provideMap),
  ],
});

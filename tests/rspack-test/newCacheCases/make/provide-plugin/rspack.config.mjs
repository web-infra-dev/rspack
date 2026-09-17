import { rspack } from '@rspack/core';

let index = 0;
let provideMap = {
  Mod: ['./a', 'default'],
};

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  entry: './entry.js',
  optimization: {
    minimize: false,
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    {
      apply(_compiler) {
        if (index === 1) {
          provideMap.Mod[0] = './b';
        }
        index++;
      },
    },
    new rspack.ProvidePlugin(provideMap),
  ],
};

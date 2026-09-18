import fs from 'node:fs';
import path from 'node:path';

let MAIN;

/** @type {import("@rspack/core").Configuration} */
export default {
  entry() {
    if (fs.existsSync(MAIN)) {
      return {
        shared: './shared.js',
        main: {
          import: './main.js',
          dependOn: 'shared',
        },
      };
    }
    return {
      shared: './shared.js',
    };
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    {
      apply(compiler) {
        MAIN = path.join(compiler.context, 'main.js');

        compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
          if (!fs.existsSync(MAIN)) {
            compilation.missingDependencies.add(MAIN);
          }
        });
      },
    },
  ],
};

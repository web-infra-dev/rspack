'use strict';

const path = require('path');
const fs = require('fs');
const {
  Compilation,
  sources: { RawSource },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  output: {
    module: true,
  },
  target: 'web',
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('Test', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'TestCopyPlugin',
              stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
            },
            () => {
              const files = [
                'file.text',
                'file.json',
                'file.js',
                'file.css',
                'file.html',
              ];

              for (const file of files) {
                const testFile = path.resolve(__dirname, file);
                const content = fs.readFileSync(testFile);

                compilation.emitAsset(file, new RawSource(content));
              }
            },
          );
        });
      },
    },
  ],
};

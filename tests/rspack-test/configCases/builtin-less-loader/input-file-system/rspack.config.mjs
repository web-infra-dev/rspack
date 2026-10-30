import assert from 'node:assert';
import fs from 'node:fs';
import path from 'node:path';

/** @type {import('@rspack/core').Configuration} */
export default {
  resolve: {
    alias: {
      'virtual-theme': path.join(import.meta.dirname, 'virtual-theme.less'),
    },
  },
  module: {
    rules: [
      { test: /\.less$/, type: 'asset/source', use: 'builtin:less-loader' },
    ],
  },
  experiments: { useInputFileSystem: [/virtual-theme\.less$/] },
  plugins: [
    {
      apply(compiler) {
        let reads = 0;
        compiler.inputFileSystem = {
          ...fs,
          readFile(file, callback) {
            if (String(file).endsWith('virtual-theme.less')) {
              reads++;
              callback(null, Buffer.from('@color: purple;'));
            } else {
              fs.readFile(file, callback);
            }
          },
          stat(file, callback) {
            fs.stat(
              String(file).endsWith('virtual-theme.less')
                ? path.join(import.meta.dirname, 'style.less')
                : file,
              callback,
            );
          },
        };
        compiler.hooks.afterCompile.tap('BuiltinLessInputFileSystem', () => {
          assert(
            reads > 0,
            'Less imports must use the configured input file system',
          );
        });
      },
    },
  ],
};

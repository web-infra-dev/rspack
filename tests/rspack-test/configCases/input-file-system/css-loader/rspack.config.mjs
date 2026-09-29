import fs from 'node:fs';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: ['css-loader'],
      },
    ],
  },
  experiments: {
    css: false,
    useInputFileSystem: [/.*/],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.inputFileSystem = {
          readFile: fs.readFile,
          stat: fs.stat,
        };
      },
    },
  ],
};

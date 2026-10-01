import memfs from 'memfs';
import fs from 'node:fs';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    (compiler) => {
      compiler.outputFileSystem = memfs.fs;
      compiler.inputFileSystem = memfs.fs;
      compiler.intermediateFileSystem = memfs.fs;

      compiler.outputFileSystem = fs;
      compiler.inputFileSystem = fs;
      compiler.intermediateFileSystem = fs;
    },
  ],
};

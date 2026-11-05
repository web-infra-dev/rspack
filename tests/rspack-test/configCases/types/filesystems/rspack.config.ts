import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import memfs from 'memfs';
import fs from 'node:fs';

export default defineConfig({
  plugins: [
    (compiler: Compiler) => {
      // memfs implements these filesystems, but its overloads differ from Node types.
      compiler.outputFileSystem =
        memfs.fs as unknown as Compiler['outputFileSystem'];
      compiler.inputFileSystem =
        memfs.fs as unknown as Compiler['inputFileSystem'];
      compiler.intermediateFileSystem =
        memfs.fs as unknown as Compiler['intermediateFileSystem'];

      compiler.outputFileSystem = fs;
      compiler.inputFileSystem = fs;
      compiler.intermediateFileSystem = fs;
    },
  ],
});

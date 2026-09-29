import fs from 'node:fs';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      compiler.outputFileSystem = fs;
      compiler.inputFileSystem = fs;
      compiler.intermediateFileSystem = fs;
    }),
  ],
});

import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  incremental: false,
  devtool: 'source-map',
  module: { rules: [{ test: /importer\.js$/, use: ['./loader.cjs'] }] },
  plugins: [
    (compiler: Compiler) => {
      compiler.hooks.afterCompile.tap('ChangeGeneratedInput', () => {
        const marker = path.join(compiler.context, 'mutate-input');
        if (!fs.existsSync(marker)) return;
        fs.unlinkSync(marker);
        const input = path.join(compiler.context, 'generated.txt');
        fs.writeFileSync(input, 'new\n');
        const timestamp = new Date(Date.now() + 20000);
        fs.utimesSync(input, timestamp, timestamp);
      });
    },
  ],
});

import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  experiments: { incremental: false },
  devtool: 'source-map',
  module: { rules: [{ test: /importer\.js$/, use: ['./loader.cjs'] }] },
  plugins: [
    (compiler) => {
      compiler.hooks.afterCompile.tap('ChangeGeneratedInput', (compilation) => {
        const input = path.join(compiler.context, 'generated.txt');
        compilation.fileDependencies.add(input);
        const extra = path.join(compiler.context, 'plugin-only.txt');
        if (fs.existsSync(extra)) compilation.fileDependencies.add(extra);
        const marker = path.join(compiler.context, 'mutate-input');
        if (!fs.existsSync(marker)) return;
        fs.unlinkSync(marker);
        fs.writeFileSync(input, 'new\n');
        const timestamp = new Date(Date.now() + 20000);
        fs.utimesSync(input, timestamp, timestamp);
      });
    },
  ],
});

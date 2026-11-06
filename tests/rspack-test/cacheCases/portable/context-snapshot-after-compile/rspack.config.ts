import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  plugins: [
    (compiler: Compiler) => {
      compiler.hooks.afterCompile.tap('GenerateContextChild', (compilation) => {
        const dir = path.join(compiler.context, 'dir');
        compilation.contextDependencies.add(dir);
        const marker = path.join(compiler.context, 'generate');
        if (!fs.existsSync(marker)) return;
        const name = fs.readFileSync(marker, 'utf8');
        fs.unlinkSync(marker);
        fs.writeFileSync(
          path.join(dir, `${name}.js`),
          `module.exports = ${JSON.stringify(name)};`,
        );
      });
    },
  ],
});

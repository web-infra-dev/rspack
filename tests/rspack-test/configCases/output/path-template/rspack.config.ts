import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';
import fs from 'node:fs';

export default defineConfig((_, { testPath }) => ({
  output: {
    path: path.join(testPath, '__[fullhash]__'),
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('Test', () => {
          const dirs = fs.readdirSync(testPath);
          expect(dirs).not.toContain('__[fullhash]__');
        });
      },
    }),
  ],
}));

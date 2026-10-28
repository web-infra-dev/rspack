import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import path from 'node:path';

class Plugin {
  apply(compiler: Compiler) {
    {
      const normalResolver1 = compiler.resolverFactory.get('normal');
      const normalResolver2 = compiler.resolverFactory.get('normal');
      expect(normalResolver1 === normalResolver2).toBe(true);
    }
    {
      const normalResolver1 = compiler.resolverFactory.get('normal', {
        alias: {
          foo: path.resolve(import.meta.dirname, 'index.js'),
        },
      });
      const normalResolver2 = compiler.resolverFactory.get('normal', {
        alias: {
          foo: path.resolve(import.meta.dirname, 'index.js'),
        },
      });
      expect(normalResolver1 === normalResolver2).toBe(true);
    }
    {
      const normalResolver1 = compiler.resolverFactory.get('normal', {
        alias: {
          foo: path.resolve(import.meta.dirname, 'index.js'),
        },
      });
      const normalResolver2 = compiler.resolverFactory.get('normal', {
        alias: {
          bar: path.resolve(import.meta.dirname, 'index.js'),
        },
      });
      expect(normalResolver1 != normalResolver2).toBe(true);
    }
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
});

import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import path from 'node:path';

class Plugin {
  apply(compiler: Compiler) {
    {
      const normalResolver1 = compiler.resolverFactory.get('normal');
      const child1 = normalResolver1.withOptions({
        alias: {
          foo: path.resolve(import.meta.dirname, 'index.js'),
        },
      });

      const normalResolver2 = compiler.resolverFactory.get('normal');
      const child2 = normalResolver2.withOptions({
        alias: {
          foo: path.resolve(import.meta.dirname, 'index.js'),
        },
      });

      expect(child1 === child2).toBe(true);
    }
    {
      const normalResolver1 = compiler.resolverFactory.get('normal', {
        alias: {
          foo: path.resolve(import.meta.dirname, 'index.js'),
        },
      });
      const child1 = normalResolver1.withOptions({});

      const normalResolver2 = compiler.resolverFactory.get('normal');
      const child2 = normalResolver2.withOptions({
        alias: {
          foo: path.resolve(import.meta.dirname, 'index.js'),
        },
      });

      expect(child1 === child2).toBe(true);
    }
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
});

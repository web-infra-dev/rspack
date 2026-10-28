import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import path from 'node:path';

class Plugin {
  apply(compiler: Compiler) {
    {
      const normalResolver = compiler.resolverFactory.get('normal');
      const request = normalResolver.resolveSync(
        {},
        import.meta.dirname,
        'foo',
      );
      expect(request).toBe(path.join(import.meta.dirname, 'index.js'));
    }
    {
      const normalResolver = compiler.resolverFactory.get('normal', {
        alias: {
          bar: path.resolve(import.meta.dirname, 'index.js'),
        },
      });
      const request = normalResolver.resolveSync(
        {},
        import.meta.dirname,
        'bar',
      );
      expect(request).toBe(path.join(import.meta.dirname, 'index.js'));
    }
  }
}

export default defineConfig({
  entry: './index.js',
  resolve: {
    alias: {
      foo: path.resolve(import.meta.dirname, 'index.js'),
    },
  },
  plugins: [new Plugin()],
});

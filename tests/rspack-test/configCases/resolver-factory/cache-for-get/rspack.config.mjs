import path from 'node:path';

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
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

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  plugins: [new Plugin()],
};

import path from 'node:path';

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
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

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  resolve: {
    alias: {
      foo: path.resolve(import.meta.dirname, 'index.js'),
    },
  },
  plugins: [new Plugin()],
};

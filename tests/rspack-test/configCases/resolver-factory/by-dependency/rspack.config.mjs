import path from 'node:path';

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    {
      const normalResolver = compiler.resolverFactory.get('normal', {
        dependencyType: 'commonjs',
      });
      expect(() =>
        normalResolver.resolveSync({}, import.meta.dirname, 'foo'),
      ).toThrow('NotFound("foo")');
    }
    {
      const normalResolver = compiler.resolverFactory.get('normal', {
        dependencyType: 'esm',
      });
      const request = normalResolver.resolveSync(
        {},
        import.meta.dirname,
        'foo',
      );
      expect(request).toBe(path.join(import.meta.dirname, 'index.js'));
    }
  }
}

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  resolve: {
    byDependency: {
      esm: {
        alias: {
          foo: path.resolve(import.meta.dirname, 'index.js'),
        },
      },
    },
  },
  plugins: [new Plugin()],
};

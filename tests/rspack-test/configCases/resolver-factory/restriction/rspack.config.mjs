import path from 'node:path';

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    const normalResolver = compiler.resolverFactory.get('normal');
    const newResolver = normalResolver.withOptions({
      extensions: ['.css'],
      mainFields: ['main'],
      restrictions: [/\.css$/],
    });
    const request = newResolver.resolveSync({}, import.meta.dirname, 'style');
    expect(request).toBe(
      path.join(import.meta.dirname, '/node_modules/style/index.css'),
    );
  }
}

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  plugins: [new Plugin()],
};

import { resolve, normalize } from 'node:path';

class Plugin {
  apply(compiler) {
    compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
      const entry = compilation.entries.get('main');
      const entryDependency = entry.dependencies[0];
      const entryModule = compilation.moduleGraph.getModule(entryDependency);
      expect(normalize(entryModule.matchResource)).toEqual(
        resolve(import.meta.dirname, 'index.js'),
      );
    });
  }
}

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  entry: {
    main: './index.js!=!./loader.mjs',
  },
  plugins: [new Plugin()],
};

import { normalize } from 'node:path';

class Plugin {
  apply(compiler) {
    compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
      const entry = compilation.entries.get('main');
      const entryDependency = entry.dependencies[0];
      const entryModule = compilation.moduleGraph.getModule(entryDependency);
      expect(
        normalize(
          entryModule.libIdent({
            context: import.meta.dirname,
          }),
        ),
      ).toEqual('index.js');
    });
  }
}

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  entry: {
    main: './index.js',
  },
  plugins: [new Plugin()],
};

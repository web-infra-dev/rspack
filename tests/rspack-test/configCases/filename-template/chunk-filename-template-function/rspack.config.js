const path = require('path');

const entryFilename = () => 'entry-[name].js';
const sharedFilename = () => 'shared-[name].js';
const addEntryFilename = () => 'add-entry-[name].js';
const addIncludeFilename = () => 'add-include-[name].js';

const pluginName = 'chunk-filename-template-function';

class Plugin {
  apply(compiler) {
    const { EntryPlugin } = compiler.rspack;
    const addEntryDependency = EntryPlugin.createDependency(
      path.resolve(__dirname, './add-entry.js'),
    );
    const addIncludeDependency = EntryPlugin.createDependency(
      path.resolve(__dirname, './add-include.js'),
    );

    compiler.hooks.make.tapPromise(
      pluginName,
      (compilation) =>
        new Promise((resolve, reject) => {
          compilation.addEntry(
            compiler.context,
            addEntryDependency,
            {
              name: 'add-entry',
              filename: addEntryFilename,
            },
            (error) => (error ? reject(error) : resolve()),
          );
        }),
    );
    compiler.hooks.finishMake.tapPromise(
      pluginName,
      (compilation) =>
        new Promise((resolve, reject) => {
          compilation.addInclude(
            compiler.context,
            addIncludeDependency,
            {
              name: 'add-include',
              filename: addIncludeFilename,
            },
            (error) => (error ? reject(error) : resolve()),
          );
        }),
    );

    let checked = false;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.afterSeal.tap(pluginName, () => {
        const chunks = Object.fromEntries(
          [...compilation.chunks].map((chunk) => [chunk.name, chunk]),
        );

        expect(chunks.main.filenameTemplate).toBe(entryFilename);
        expect(chunks.shared.filenameTemplate).toBe(sharedFilename);
        expect(chunks['add-entry'].filenameTemplate).toBe(addEntryFilename);
        expect(chunks['add-include'].filenameTemplate).toBe(addIncludeFilename);
        expect(chunks.async.filenameTemplate).toBeUndefined();

        checked = true;
      });
    });
    compiler.hooks.done.tap(pluginName, (stats) => {
      expect(stats.toJson().errors).toHaveLength(0);
      expect(checked).toBe(true);
    });
  }
}

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  context: __dirname,
  mode: 'development',
  target: 'node',
  entry: {
    main: {
      import: './index.js',
      filename: entryFilename,
    },
  },
  output: {
    chunkFilename: 'async-[name].js',
  },
  optimization: {
    chunkIds: 'named',
    splitChunks: {
      cacheGroups: {
        shared: {
          chunks: 'all',
          test: /shared/,
          name: 'shared',
          filename: sharedFilename,
          enforce: true,
        },
      },
    },
  },
  plugins: [new Plugin()],
};

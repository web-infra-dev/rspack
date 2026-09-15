const { javascript } = require('@rspack/core');

const pluginName = 'plugin';

class Plugin {
  apply(compiler) {
    let called = false;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.afterSeal.tap(pluginName, () => {
        called = true;

        const templates = {};
        for (const chunk of compilation.chunks) {
          const template =
            javascript.JavascriptModulesPlugin.getChunkFilenameTemplate(
              chunk,
              compilation.outputOptions,
            );
          templates[chunk.name || chunk.id] = template;

          // The returned template has to be the one rspack actually rendered
          // the chunk with.
          expect([...chunk.files]).toContain(
            compilation.getPath(template, {
              chunk,
              contentHashType: 'javascript',
            }),
          );
        }

        expect(templates).toEqual({
          // initial chunk without its own template -> output.filename
          main: '[name].js',
          // async chunk without its own template -> output.chunkFilename
          async_js: 'async-[name].js',
          // chunk carrying its own template -> that template wins
          'shared-shared_js': 'shared-[name].js',
        });
      });
    });
    compiler.hooks.done.tap(pluginName, (stats) => {
      expect(stats.toJson().errors.length).toBe(0);
      expect(called).toBe(true);
    });
  }
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  context: __dirname,
  mode: 'development',
  entry: './index.js',
  target: 'node',
  output: {
    filename: '[name].js',
    chunkFilename: 'async-[name].js',
  },
  optimization: {
    chunkIds: 'named',
    splitChunks: {
      cacheGroups: {
        shared: {
          chunks: 'all',
          test: /shared/,
          filename: 'shared-[name].js',
          enforce: true,
        },
      },
    },
  },
  plugins: [new Plugin()],
};

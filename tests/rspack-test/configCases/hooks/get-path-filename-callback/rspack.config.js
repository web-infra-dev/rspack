const pluginName = 'plugin';

class Plugin {
  apply(compiler) {
    let called = false;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.processAssets.tap(pluginName, () => {
        called = true;
        const chunk = Array.from(compilation.chunks).find(
          (c) => c.name === 'main',
        );
        expect(chunk).toBeDefined();

        // a string template keeps working
        expect(compilation.getPath('[name].js', { chunk })).toBe('main.js');

        // a callback is evaluated, and placeholders it returns are still rendered
        expect(
          compilation.getPath(() => 'from-callback-[name].js', { chunk }),
        ).toBe('from-callback-main.js');

        // the callback receives the path data it was called with
        let seen;
        compilation.getPath(
          (pathData) => {
            seen = pathData;
            return '[name].js';
          },
          { chunk, contentHashType: 'javascript' },
        );
        expect(seen.chunk).toBe(chunk);
        expect(seen.contentHashType).toBe('javascript');

        // the other three helpers accept callbacks too
        expect(compilation.getAssetPath(() => '[name].js', { chunk })).toBe(
          'main.js',
        );
        expect(
          compilation.getPathWithInfo(() => '[name].js', { chunk }).path,
        ).toBe('main.js');
        expect(
          compilation.getAssetPathWithInfo(() => '[name].js', { chunk }).path,
        ).toBe('main.js');
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
  plugins: [new Plugin()],
};

const fs = require('node:fs');
const path = require('node:path');
const checkMap =
  require('@rspack/test-tools/helper/util/checkSourceMap').default;

class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.done.tapAsync('Plugin', (stats, callback) => {
      (async () => {
        const outputPath = stats.compilation.getPath(compiler.outputPath, {});
        const source = fs.readFileSync(
          path.resolve(outputPath, 'bundle0.css.map'),
          'utf-8',
        );
        const map = JSON.parse(source);
        const fooSource = map.sources.find((source) =>
          source.endsWith('foo.scss'),
        );
        const barSource = map.sources.find((source) =>
          source.endsWith('bar.scss'),
        );

        expect(fooSource).toBeTruthy();
        expect(barSource).toBeTruthy();
        expect(map.sources).not.toEqual(
          expect.arrayContaining([
            expect.stringContaining('builtin:lightningcss-loader'),
          ]),
        );
        expect(map.file).toEqual('bundle0.css');
        const normalizedBarSource = `webpack:///${path.basename(barSource)}`;

        const css = fs.readFileSync(
          path.resolve(outputPath, 'bundle0.css'),
          'utf-8',
        );
        expect(
          await checkMap(css, source, {
            '.child {': {
              inSource: normalizedBarSource,
              outId: '.bar .child {',
            },
          }),
        ).toBe(true);
      })().then(() => callback(), callback);
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.scss$/,
        use: [
          'builtin:lightningcss-loader',
          {
            loader: 'sass-loader',
            options: {
              sassOptions: {
                style: 'expanded',
              },
            },
          },
        ],
        type: 'css/auto',
      },
    ],
  },
  plugins: [new Plugin()],
};

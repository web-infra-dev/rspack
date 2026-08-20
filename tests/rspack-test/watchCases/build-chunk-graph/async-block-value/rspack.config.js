const fs = require('fs');
const path = require('path');

const PLUGIN_NAME = 'CheckAsyncBlockOriginsPlugin';

class CheckAsyncBlockOriginsPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.afterProcessAssets.tap(PLUGIN_NAME, () => {
        const source = fs.readFileSync(
          path.join(compilation.options.context, 'index.js'),
          'utf-8',
        );
        const expectedLines = new Map(
          ['./a', './b'].map((request) => {
            const offset = source.indexOf(`import("${request}")`);
            expect(offset).toBeGreaterThanOrEqual(0);
            return [request, source.slice(0, offset).split('\n').length];
          }),
        );
        const asyncOrigins = compilation.chunkGroups
          .flatMap((chunkGroup) => chunkGroup.origins)
          .filter((origin) => expectedLines.has(origin.request));

        expect(asyncOrigins).toHaveLength(2);
        for (const origin of asyncOrigins) {
          expect(origin.loc.start.line).toBe(expectedLines.get(origin.request));
        }
      });
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    splitChunks: false,
  },
  plugins: [new CheckAsyncBlockOriginsPlugin()],
};

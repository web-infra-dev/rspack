const path = require('path');
const fs = require('fs');

function normalizeToUrlStyle(s) {
  const withForward = s.replace(/\\/g, '/');
  return path.posix.normalize(withForward);
}

function formatSources(sources) {
  return sources.map((s) => `  - ${s}`).join('\n');
}

function readInlineSourceMap(assetPath) {
  const source = fs.readFileSync(assetPath, 'utf-8');
  const match = source.match(
    /\/\/# sourceMappingURL=data:application\/json;charset=utf-8;base64,([A-Za-z0-9+/=]+)\s*$/,
  );

  expect(
    match,
    `${assetPath} should contain an inline source map`,
  ).toBeTruthy();
  return JSON.parse(Buffer.from(match[1], 'base64').toString('utf-8'));
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: 'inline-source-map',
  entry: {
    shallow: {
      import: './src/index.js',
      filename: 'static/js/[name].js',
    },
    deep: {
      import: './src/index.js',
      filename: 'static/js/nested/[name].js',
    },
  },
  output: {
    filename: 'static/js/[name].js',
    devtoolModuleFilenameTemplate: '[relative-resource-path]',
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('PLUGIN', (stats) => {
          const outputPath = stats.compilation.getPath(compiler.outputPath, {});
          const expectedFiles = [
            path.resolve(__dirname, 'src/index.js'),
            path.resolve(__dirname, 'src/button/index.js'),
          ].sort();

          const assertInlineSourceMapSources = (assetFilename) => {
            const assetPath = path.join(outputPath, assetFilename);
            const sourceMap = readInlineSourceMap(assetPath);
            const realSources = sourceMap.sources
              .filter((s) => !s.startsWith('webpack://'))
              .sort();

            realSources.forEach((s) => {
              expect(
                path.isAbsolute(s),
                `${assetFilename} contains an absolute source path:\n  - ${s}`,
              ).toBe(false);
              expect(
                normalizeToUrlStyle(s),
                `${assetFilename} contains a non-normalized source path:\n  - ${s}`,
              ).toBe(s);
            });

            const assetDir = path.dirname(assetPath);
            const expectedSources = expectedFiles
              .map((file) => normalizeToUrlStyle(path.relative(assetDir, file)))
              .sort();

            expect(
              realSources.join('\n'),
              [
                `${assetFilename} should contain sources relative to its own directory.`,
                'Expected sources:',
                formatSources(expectedSources),
                'Actual sources:',
                formatSources(realSources),
              ].join('\n'),
            ).toBe(expectedSources.join('\n'));
          };

          assertInlineSourceMapSources('static/js/shallow.js');
          assertInlineSourceMapSources('static/js/nested/deep.js');
        });
      },
    },
  ],
};

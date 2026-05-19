const path = require('path');
const fs = require('fs');

function normalizeToUrlStyle(s) {
  // 1) Convert Windows backslashes to forward slashes
  const withForward = s.replace(/\\/g, '/');
  // 2) POSIX-normalize to collapse ".." / "." segments
  return path.posix.normalize(withForward);
}

function formatSources(sources) {
  return sources.map((s) => `  - ${s}`).join('\n');
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: 'source-map',
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

          const assertSourceMapSources = (sourceMapFilename) => {
            const sourceMapPath = path.join(outputPath, sourceMapFilename);
            const sourceMapJSON = fs.readFileSync(sourceMapPath, 'utf-8');
            const sourceMap = JSON.parse(sourceMapJSON);
            const realSources = sourceMap.sources
              .filter((s) => !s.startsWith('webpack://'))
              .sort();

            realSources.forEach((s) => {
              expect(
                path.isAbsolute(s),
                `${sourceMapFilename} contains an absolute source path:\n  - ${s}`,
              ).toBe(false);
              expect(
                normalizeToUrlStyle(s),
                `${sourceMapFilename} contains a non-normalized source path:\n  - ${s}`,
              ).toBe(s);
            });

            const mapDir = path.dirname(sourceMapPath);
            const expectedSources = expectedFiles
              .map((file) => normalizeToUrlStyle(path.relative(mapDir, file)))
              .sort();

            expect(
              realSources.join('\n'),
              [
                `${sourceMapFilename} should contain sources relative to its own directory.`,
                'Expected sources:',
                formatSources(expectedSources),
                'Actual sources:',
                formatSources(realSources),
              ].join('\n'),
            ).toBe(expectedSources.join('\n'));
          };

          assertSourceMapSources('static/js/shallow.js.map');
          assertSourceMapSources('static/js/nested/deep.js.map');
        });
      },
    },
  ],
};

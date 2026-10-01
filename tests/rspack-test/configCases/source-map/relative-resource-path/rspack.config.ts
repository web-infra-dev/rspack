import type { RawSourceMap } from 'source-map';
import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';
import fs from 'node:fs';

function normalizeToUrlStyle(s: string) {
  // 1) Convert Windows backslashes to forward slashes
  const withForward = s.replace(/\\/g, '/');
  // 2) POSIX-normalize to collapse ".." / "." segments
  return path.posix.normalize(withForward);
}

function formatSources(sources: string[]) {
  return sources.map((s) => `  - ${s}`).join('\n');
}

export default defineConfig({
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
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('PLUGIN', (stats) => {
          const outputPath = stats.compilation.getPath(compiler.outputPath, {});
          const expectedFiles = [
            path.resolve(import.meta.dirname, 'src/index.js'),
            path.resolve(import.meta.dirname, 'src/button/index.js'),
          ].sort();

          const assertSourceMapSources = (sourceMapFilename: string) => {
            const sourceMapPath = path.join(outputPath, sourceMapFilename);
            const sourceMapJSON = fs.readFileSync(sourceMapPath, 'utf-8');
            const sourceMap: RawSourceMap = JSON.parse(sourceMapJSON);
            const realSources = sourceMap.sources
              .filter(
                (s) =>
                  !s.startsWith('webpack://') && !s.startsWith('rspack://'),
              )
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
    }),
  ],
});

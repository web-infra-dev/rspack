import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import checkMap from '@rspack/test-tools/helper/util/checkSourceMap';
import type { RawSourceMap } from 'source-map';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.done.tapAsync('Plugin', (stats, callback) => {
      (async () => {
        const outputPath = stats.compilation.getPath(compiler.outputPath, {});
        const source = fs.readFileSync(
          path.resolve(outputPath, 'bundle0.css.map'),
          'utf-8',
        );
        const map: RawSourceMap = JSON.parse(source);
        const fooSource = map.sources.find((source) =>
          source.endsWith('foo.scss'),
        );
        const barSource = map.sources.find((source) =>
          source.endsWith('bar.scss'),
        );

        expect(fooSource).toBeTruthy();
        expect(barSource).toBeTruthy();
        expect(map.file).toEqual('bundle0.css');
        let normalizedBarSource = `webpack:///${path.basename(barSource!)}`;
        if (compiler.options.experiments?.runtimeMode === 'rspack') {
          normalizedBarSource = `rspack:///${path.basename(barSource!)}`;
        }

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

export default defineConfig({
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
});

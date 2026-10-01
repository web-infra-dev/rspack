import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  Compilation,
  sources: { RawSource },
} = rspack;

export default defineConfig({
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  target: ['web', 'es2020'],
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('html-plugin', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'copy-plugin',
              stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
            },
            () => {
              [
                'static-package.json',
                'static-package-str.json',
                'dynamic-package.json',
                'dynamic-package-str.json',
                'eager.json',
                'weak.json',
                './nested/pkg.json',
                're-export.json',
                're-export-directly.json',
              ].forEach((filename) => {
                const resolvedFilename = path.resolve(
                  import.meta.dirname,
                  filename,
                );
                const content = fs.readFileSync(resolvedFilename);
                compilation.emitAsset(
                  filename.replace(/\.\/nested\//, ''),
                  new RawSource(content),
                );
              });

              const content = compilation
                .getAsset('bundle0.mjs')
                ?.source.source()
                .toString();
              expect(content?.replace(/[ \t]+$/gm, '')).toMatchFileSnapshotSync(
                path.join(
                  import.meta.dirname,
                  '__snapshots__',
                  'bundle0.mjs.txt',
                ),
              );
            },
          );
        });
      },
    }),
  ],
  externals: {
    './static-package.json': 'module ./static-package.json',
    './static-package-str.json': 'module ./static-package-str.json',
    './dynamic-package.json': 'import ./dynamic-package.json',
    './dynamic-package-str.json': 'import ./dynamic-package-str.json',
    './eager.json': 'import ./eager.json',
    './weak.json': 'import ./weak.json',
    './pkg.json': 'import ./pkg.json',
    './pkg': 'import ./pkg.json',
    './re-export.json': 'module ./re-export.json',
    './re-export-directly.json': 'module ./re-export-directly.json',
  },
});

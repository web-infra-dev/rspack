import assert from 'node:assert/strict';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

export default ['a.json', 'references.json'].map((configFile, index) =>
  defineConfig({
    output: { filename: `bundle${index}.js` },
    plugins: [
      {
        apply(compiler: Compiler) {
          compiler.options.resolve.tsConfig = {
            configFile: path.join(compiler.context, configFile),
            references: 'auto',
          };
          compiler.hooks.afterCompile.tap(
            'CheckTsconfigDependencies',
            (compilation) => {
              for (const file of new Set([configFile, 'a.json', 'b.json'])) {
                assert(
                  compilation.fileDependencies.has(
                    path.join(compiler.context, file),
                  ),
                  `Missing tsconfig dependency: ${file}`,
                );
              }
            },
          );
        },
      },
    ],
  }),
);

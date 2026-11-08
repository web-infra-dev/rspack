import { defineConfig } from '@rspack/cli';
import { type Compiler } from '@rspack/core';
import path from 'node:path';
import fs from 'node:fs';

const pluginName = 'plugin';

class Plugin {
  sourcePath: string;
  outputPath: string;

  constructor(options: { sourcePath: string; outputPath: string }) {
    this.sourcePath = options.sourcePath;
    this.outputPath = options.outputPath;
  }

  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(
      pluginName,
      (_compilation, { normalModuleFactory }) => {
        normalModuleFactory.hooks.beforeResolve.tap(
          pluginName,
          (resolveData) => {
            const { request, context } = resolveData;

            if (context === this.sourcePath && request === './text.txt') {
              const sourcePath = path.resolve(this.sourcePath, request);
              const outputPath = path.resolve(this.outputPath, `${request}.js`);

              const source = fs.readFileSync(sourcePath, { encoding: 'utf-8' });
              fs.mkdirSync(path.dirname(outputPath), { recursive: true });

              const sourceMod = fs.statSync(sourcePath).mtime;

              if (
                !fs.existsSync(outputPath) ||
                fs.statSync(outputPath).mtime < sourceMod
              ) {
                fs.writeFileSync(
                  outputPath,
                  `export const text = \`${source}\``,
                );
              }

              resolveData.context = this.outputPath;

              resolveData.request = `${request}.js`;

              // console.log(resolveData);
            }

            return undefined;
          },
        );
      },
    );
  }
}

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [],
  },
  plugins: [
    new Plugin({
      sourcePath: path.resolve(import.meta.dirname),
      outputPath: path.resolve(import.meta.dirname, '.temp'),
    }),
  ],
});

import path from 'node:path';
import fs from 'node:fs';

const pluginName = 'plugin';

class Plugin {
  constructor(options) {
    this.sourcePath = options.sourcePath;
    this.outputPath = options.outputPath;
  }

  apply(compiler) {
    this.logger = compiler.getInfrastructureLogger(pluginName);

    compiler.hooks.compilation.tap(
      pluginName,
      (compilation, { normalModuleFactory }) => {
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
/**@type {import("@rspack/core").Configuration}*/
export default {
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
};

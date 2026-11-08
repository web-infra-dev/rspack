import { defineConfig, definePlugin } from '@rspack/cli';
import { type Configuration, rspack } from '@rspack/core';
import fs from 'node:fs';
import path from 'node:path';

const {
  experiments: { RsdoctorPlugin },
} = rspack;

const dataPath = path.join(import.meta.dirname, 'data.json');
const generatedSource = `module.exports = ${JSON.stringify({ used: 'kept' })}`;
const originalSourceSize =
  'module.exports = '.length +
  JSON.stringify(JSON.parse(fs.readFileSync(dataPath, 'utf-8'))).length;

export default defineConfig(
  (['source-map', 'cheap-module-source-map'] as const).map<Configuration>(
    (devtool) => ({
      mode: 'development',
      devtool,
      optimization: {
        concatenateModules: false,
        sideEffects: false,
        usedExports: true,
      },
      plugins: [
        new RsdoctorPlugin({
          moduleGraphFeatures: ['graph', 'sources'],
          chunkGraphFeatures: false,
        }),
        definePlugin({
          apply(compiler) {
            compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
              const hooks = RsdoctorPlugin.getCompilationHooks(compilation);

              hooks.moduleSources.tap('TestPlugin', ({ jsonModuleSizes }) => {
                const jsonModule = [...compilation.modules].find(
                  (module) => module.type === 'json',
                )!;
                const source = compilation.codeGenerationResults
                  .get(jsonModule, 'main')
                  .sources.get('javascript')!;

                expect(source.source().toString()).toBe(generatedSource);
                expect(source.size()).toBeLessThan(originalSourceSize);
                expect(jsonModuleSizes).toEqual([
                  {
                    identifier: jsonModule.identifier(),
                    size: source.size(),
                  },
                ]);
              });
            });
          },
        }),
      ],
    }),
  ),
);

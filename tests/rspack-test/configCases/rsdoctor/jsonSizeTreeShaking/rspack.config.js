const fs = require('fs');
const path = require('path');
const {
  experiments: { RsdoctorPlugin },
} = require('@rspack/core');

const dataPath = path.join(__dirname, 'data.json');
const generatedSource = `module.exports = ${JSON.stringify({ used: 'kept' })}`;
const originalSourceSize =
  'module.exports = '.length +
  JSON.stringify(JSON.parse(fs.readFileSync(dataPath, 'utf-8'))).length;

/** @type {import('@rspack/core').Configuration[]} */
module.exports = ['source-map', 'cheap-module-source-map'].map((devtool) => ({
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
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          const hooks = RsdoctorPlugin.getCompilationHooks(compilation);

          hooks.moduleSources.tap('TestPlugin', ({ jsonModuleSizes }) => {
            const jsonModule = [...compilation.modules].find(
              (module) => module.type === 'json',
            );
            const source = compilation.codeGenerationResults
              .get(jsonModule, 'main')
              .sources.get('javascript');

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
    },
  ],
}));

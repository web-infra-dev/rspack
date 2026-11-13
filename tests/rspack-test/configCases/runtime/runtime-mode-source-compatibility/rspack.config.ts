import { defineConfig, definePlugin } from '@rspack/cli';

export default [false, true].map((esm, index) =>
  defineConfig({
    mode: 'development',
    target: 'node',
    entry: './index.js',
    devtool: 'source-map',
    output: {
      filename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      chunkFilename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      ...(esm ? { module: true, library: { type: 'modern-module' } } : {}),
    },
    optimization: {
      minimize: false,
      concatenateModules: false,
      runtimeChunk: esm ? 'single' : false,
    },
    plugins: [
      definePlugin((compiler) => {
        const { RuntimeModule, RuntimeGlobals } = compiler.rspack;
        const rspackMode =
          compiler.options.experiments.runtimeMode === 'rspack';
        new compiler.rspack.DefinePlugin({
          RSPACK_MODE: JSON.stringify(rspackMode),
          ESM_OUTPUT: JSON.stringify(esm),
          RUNTIME_SOURCE_MAP: JSON.stringify(
            esm ? `runtime${index}.mjs.map` : `main${index}.js.map`,
          ),
        }).apply(compiler);
        // Its native runtimeModule hook separates the two JS hook stages.
        new compiler.rspack.experiments.RstestPlugin({
          injectModulePathName: false,
          importMetaPathName: false,
          hoistMockModule: false,
          manualMockRoot: import.meta.dirname,
        }).apply(compiler);

        compiler.hooks.thisCompilation.tap(
          'RuntimeSourceCompatibility',
          (compilation) => {
            const generations = new Map<string, number>();
            class CompatRuntime extends RuntimeModule {
              private code: string;
              constructor(name: string, code: string) {
                super(name, RuntimeModule.STAGE_TRIGGER);
                this.code = code;
              }
              generate() {
                generations.set(
                  this.name,
                  (generations.get(this.name) ?? 0) + 1,
                );
                return this.code;
              }
            }

            compilation.hooks.additionalTreeRuntimeRequirements.tap(
              'RuntimeSourceCompatibility',
              (chunk, requirements) => {
                requirements.add(RuntimeGlobals.require);
                for (const [name, property] of [
                  ['runtime-compat-empty', 'compatEmpty'],
                  ['runtime-compat-generate', 'compatGenerate'],
                  ['runtime-compat-stages', 'compatStages'],
                  ['runtime-compat-mapped', 'compatMapped'],
                ]) {
                  const before = generations.get(name) ?? 0;
                  compilation.addRuntimeModule(
                    chunk,
                    new CompatRuntime(
                      name,
                      `__webpack_require__.${property} = "original";`,
                    ),
                  );
                  if (
                    (generations.get(name) ?? 0) - before !==
                    Number(rspackMode)
                  ) {
                    throw new Error(
                      `Unexpected addRuntimeModule generation count for ${name}`,
                    );
                  }
                }
                for (const asyncChunk of chunk.getAllAsyncChunks()) {
                  if (asyncChunk.name === 'compat-async') {
                    compilation.addRuntimeModule(
                      asyncChunk,
                      new CompatRuntime(
                        'runtime-compat-async',
                        'globalThis.runtimeCompatAsync = true;',
                      ),
                    );
                  }
                }
              },
            );
            compilation.hooks.runtimeModule.tap('FirstSource', (module) => {
              if (!module.source) return;
              if (module.name === 'runtime-compat-empty')
                module.source.source = '';
              if (module.name === 'runtime-compat-generate') {
                module.generate = () =>
                  '__webpack_require__.compatGenerate = "overridden";';
              }
              if (module.name === 'runtime-compat-stages') {
                module.source.source +=
                  '\n__webpack_require__.compatStages += "-first";';
              }
              if (module.name === 'runtime-compat-mapped') {
                module.source.source =
                  '__webpack_require__.compatMapped = "modified";';
              }
            });
            compilation.hooks.runtimeModule.tap(
              { name: 'AfterNativeSource', stage: Infinity },
              (module) => {
                if (module.name === 'runtime-compat-stages' && module.source) {
                  module.source.source +=
                    '\n__webpack_require__.compatStages += "-second";';
                }
              },
            );
          },
        );
      }),
    ],
  }),
);

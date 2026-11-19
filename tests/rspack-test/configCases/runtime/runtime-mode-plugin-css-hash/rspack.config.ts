import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'web',
  devtool: false,
  output: {
    cssChunkFilename: '[name].[fullhash:8].[contenthash:8].css',
    publicPath: '',
  },
  module: { rules: [{ test: /\.css$/, type: 'css/auto' }] },
  optimization: { chunkIds: 'named', realContentHash: true },
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeModule, RuntimeGlobals } = compiler.rspack;
      class CssFilenameRuntime extends RuntimeModule {
        dependentHash = true;
        constructor() {
          super('plugin CSS chunk filename', RuntimeModule.STAGE_TRIGGER);
        }
        generate() {
          const child = [...this.chunk!.getAllAsyncChunks()].find(
            (chunk) => chunk.name === 'styles',
          );
          const hash = child?.contentHash?.css?.slice(0, 8);
          return `__webpack_require__.miniCssF = id => __webpack_require__.k(id);
          __webpack_require__.pluginCssHash = ${JSON.stringify(hash ?? null)};`;
        }
      }
      compiler.hooks.thisCompilation.tap('CssFilename', (compilation) => {
        compilation.hooks.runtimeRequirementInTree
          .for(RuntimeGlobals.getChunkCssFilename)
          .tap('CssFilename', (chunk) => {
            compilation.addRuntimeModule(chunk, new CssFilenameRuntime());
          });
      });
    }),
  ],
});

const { RuntimeModule } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.js',
  mode: 'development',
  devtool: false,
  optimization: {
    minimize: false,
    moduleIds: 'named',
    chunkIds: 'named',
  },
  plugins: [
    (compiler) => {
      const RuntimeGlobals = compiler.rspack.RuntimeGlobals;
      class RuntimeVariableModule extends RuntimeModule {
        constructor() {
          super('runtime-variable-compat');
        }

        generate() {
          return `
${RuntimeGlobals.publicPath} = "runtime-public/";
${RuntimeGlobals.moduleCache}.runtimeCompat = { exports: { ok: true } };
${RuntimeGlobals.require}.runtimeCompat = function() {
	return {
		publicPath: ${RuntimeGlobals.publicPath},
		cache: ${RuntimeGlobals.moduleCache}.runtimeCompat.exports.ok
	};
};
`;
        }
      }

      compiler.hooks.thisCompilation.tap(
        'RuntimeVariableCompatPlugin',
        (compilation) => {
          compilation.hooks.additionalTreeRuntimeRequirements.tap(
            'RuntimeVariableCompatPlugin',
            (chunk, runtimeRequirements) => {
              runtimeRequirements.add(RuntimeGlobals.moduleCache);
              compilation.addRuntimeModule(chunk, new RuntimeVariableModule());
            },
          );
        },
      );
    },
  ],
};

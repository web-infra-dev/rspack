const { RuntimeModule } = require('@rspack/core');

class FullHashSourceReuseRuntimeModule extends RuntimeModule {
  constructor() {
    super('full-hash-source-reuse');
    this.fullHash = true;
  }

  generate() {
    return `__webpack_require__.fullHashSourceReuse = ${JSON.stringify(this.compilation.hash)};`;
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  target: 'node',
  entry: './index.js',
  mode: 'development',
  devtool: false,
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: false,
    sideEffects: false,
    concatenateModules: false,
    usedExports: false,
    innerGraph: false,
    providedExports: false,
    runtimeChunk: {
      name: 'runtime',
    },
  },
  plugins: [
    (compiler) => {
      compiler.hooks.thisCompilation.tap(
        'FullHashSourceReuseRuntimeModulePlugin',
        (compilation) => {
          compilation.hooks.additionalTreeRuntimeRequirements.tap(
            'FullHashSourceReuseRuntimeModulePlugin',
            (chunk) => {
              compilation.addRuntimeModule(
                chunk,
                new FullHashSourceReuseRuntimeModule(),
              );
            },
          );
        },
      );
    },
  ],
};

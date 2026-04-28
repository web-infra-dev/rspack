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
  mode: 'development',
  output: {
    filename: '[name].js',
  },
  optimization: {
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

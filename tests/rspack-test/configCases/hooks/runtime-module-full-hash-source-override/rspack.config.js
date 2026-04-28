class Plugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(
      'RuntimeModuleFullHashSourceOverridePlugin',
      (compilation) => {
        compilation.hooks.runtimeModule.tap(
          'RuntimeModuleFullHashSourceOverridePlugin',
          (module) => {
            if (module.name === 'get_full_hash') {
              const originSource = module.source.source.toString('utf-8');
              module.source.source = Buffer.from(
                `${originSource}\n__webpack_require__.hookedFullHash = "override";\n`,
                'utf-8',
              );
            }
          },
        );
      },
    );
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
  plugins: [new Plugin()],
};

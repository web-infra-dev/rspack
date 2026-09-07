class CustomModuleIdPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap('CustomModuleIdPlugin', (compilation) => {
      compilation.hooks.beforeModuleIds.tap(
        'CustomModuleIdPlugin',
        (modules) => {
          for (const module of modules) {
            if (module.identifier.includes('index.js')) {
              module.id = 'custom-entry-id';
            }
          }
        },
      );
    });
  }
}

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  cache: { type: 'memory' },
  optimization: {
    moduleIds: false,
    concatenateModules: false,
    inlineExports: false,
  },
  plugins: [new CustomModuleIdPlugin()],
  incremental: true,
};

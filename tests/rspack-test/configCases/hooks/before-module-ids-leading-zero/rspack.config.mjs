class LeadingZeroModuleIdPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(
      'LeadingZeroModuleIdPlugin',
      (compilation) => {
        compilation.hooks.beforeModuleIds.tap(
          'LeadingZeroModuleIdPlugin',
          (modules) => {
            for (const module of modules) {
              if (module.resource?.endsWith('lazy-a.js')) {
                module.id = '0795';
              }
            }
          },
        );
      },
    );
  }
}

/** @type {import('@rspack/core').Configuration} */
export default {
  target: 'node',
  mode: 'production',
  optimization: {
    minimize: false,
  },
  plugins: [new LeadingZeroModuleIdPlugin()],
};

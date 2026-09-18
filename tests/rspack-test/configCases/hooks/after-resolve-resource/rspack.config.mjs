const pluginName = 'plugin';

/**@type {import("@rspack/core").Configuration}*/
export default [
  {
    context: import.meta.dirname,
    entry: './resource.js',
    output: {
      filename: 'resource.js',
    },
    optimization: {
      moduleIds: 'named',
      concatenateModules: false,
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.compilation.tap(
            pluginName,
            (compilation, { normalModuleFactory }) => {
              normalModuleFactory.hooks.afterResolve.tap(
                pluginName,
                (resolveData) => {
                  resolveData.createData.resource =
                    resolveData.createData.resource.replace('b.js', 'c.js');
                },
              );
            },
          );
        },
      },
    ],
  },
  {
    context: import.meta.dirname,
    entry: './request.js',
    output: {
      filename: 'request.js',
    },
    optimization: {
      moduleIds: 'named',
      concatenateModules: false,
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.compilation.tap(
            pluginName,
            (compilation, { normalModuleFactory }) => {
              normalModuleFactory.hooks.afterResolve.tap(
                pluginName,
                (resolveData) => {
                  resolveData.createData.request =
                    resolveData.createData.request.replace('b.js', 'c.js');
                  resolveData.createData.userRequest =
                    resolveData.createData.userRequest.replace('b.js', 'c.js');
                },
              );
            },
          );
        },
      },
    ],
  },
  {
    context: import.meta.dirname,
    entry: './duplicate.js',
    output: {
      filename: 'duplicate.js',
    },
    optimization: {
      moduleIds: 'named',
      concatenateModules: false,
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.compilation.tap(
            pluginName,
            (compilation, { normalModuleFactory }) => {
              normalModuleFactory.hooks.afterResolve.tap(
                pluginName,
                (resolveData) => {
                  resolveData.createData.request =
                    resolveData.createData.request.replace('b.js', 'c.js');
                  resolveData.createData.userRequest =
                    resolveData.createData.userRequest.replace('b.js', 'c.js');
                  resolveData.createData.resource =
                    resolveData.createData.resource.replace('b.js', 'c.js');
                },
              );
            },
          );
        },
      },
    ],
  },
];

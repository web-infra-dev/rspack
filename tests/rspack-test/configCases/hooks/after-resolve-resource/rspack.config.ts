import { defineConfig, definePlugin } from '@rspack/cli';

const pluginName = 'plugin';

export default defineConfig([
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
      definePlugin({
        apply(compiler) {
          compiler.hooks.compilation.tap(
            pluginName,
            (_compilation, { normalModuleFactory }) => {
              normalModuleFactory.hooks.afterResolve.tap(
                pluginName,
                (resolveData) => {
                  resolveData.createData!.resource =
                    resolveData.createData!.resource.replace('b.js', 'c.js');
                },
              );
            },
          );
        },
      }),
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
      definePlugin({
        apply(compiler) {
          compiler.hooks.compilation.tap(
            pluginName,
            (_compilation, { normalModuleFactory }) => {
              normalModuleFactory.hooks.afterResolve.tap(
                pluginName,
                (resolveData) => {
                  resolveData.createData!.request =
                    resolveData.createData!.request.replace('b.js', 'c.js');
                  resolveData.createData!.userRequest =
                    resolveData.createData!.userRequest.replace('b.js', 'c.js');
                },
              );
            },
          );
        },
      }),
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
      definePlugin({
        apply(compiler) {
          compiler.hooks.compilation.tap(
            pluginName,
            (_compilation, { normalModuleFactory }) => {
              normalModuleFactory.hooks.afterResolve.tap(
                pluginName,
                (resolveData) => {
                  resolveData.createData!.request =
                    resolveData.createData!.request.replace('b.js', 'c.js');
                  resolveData.createData!.userRequest =
                    resolveData.createData!.userRequest.replace('b.js', 'c.js');
                  resolveData.createData!.resource =
                    resolveData.createData!.resource.replace('b.js', 'c.js');
                },
              );
            },
          );
        },
      }),
    ],
  },
  {
    context: import.meta.dirname,
    entry: './batch.js',
    output: {
      filename: 'batch.js',
    },
    optimization: {
      moduleIds: 'named',
      concatenateModules: false,
    },
    plugins: [
      definePlugin({
        apply(compiler) {
          let active = 0;
          let maxActive = 0;
          let matched = 0;
          compiler.hooks.compilation.tap(
            pluginName,
            (_compilation, { normalModuleFactory }) => {
              normalModuleFactory.hooks.afterResolve.tapPromise(
                pluginName,
                async (resolveData) => {
                  if (
                    !resolveData.createData!.resource.includes('b.js?batch=')
                  ) {
                    return;
                  }
                  matched++;
                  active++;
                  maxActive = Math.max(maxActive, active);
                  await new Promise((resolve) => setImmediate(resolve));
                  resolveData.createData!.resource =
                    resolveData.createData!.resource.replace('b.js', 'c.js');
                  active--;
                },
              );
            },
          );
          compiler.hooks.afterCompile.tap(pluginName, () => {
            if (matched !== 16 || maxActive < 2) {
              throw new Error(
                `Expected 16 concurrent afterResolve calls, got ${matched} calls with ${maxActive} active`,
              );
            }
          });
        },
      }),
    ],
  },
]);

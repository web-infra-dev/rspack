import assert from 'node:assert/strict';
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
                  assert(resolveData.createData);
                  resolveData.createData.resource =
                    resolveData.createData.resource.replace('b.js', 'c.js');
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
                  assert(resolveData.createData);
                  resolveData.createData.request =
                    resolveData.createData.request.replace('b.js', 'c.js');
                  resolveData.createData.userRequest =
                    resolveData.createData.userRequest.replace('b.js', 'c.js');
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
                  assert(resolveData.createData);
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
      }),
    ],
  },
]);

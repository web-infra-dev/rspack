import { container, type Compiler, NormalModule } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const { ModuleFederationPlugin } = container;
const pluginName = 'AssertMixedRuntimeConnectionPlugin';

class AssertMixedRuntimeConnectionPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.thisCompilation.tap(pluginName, (compilation) => {
      compilation.hooks.processAssets.tap(pluginName, () => {
        const runtimePlugin = Array.from(compilation.modules).find(
          (module) =>
            module instanceof NormalModule &&
            module.resource.endsWith('runtime-plugin.js'),
        );
        const connection = runtimePlugin
          ? compilation.moduleGraph
              .getOutgoingConnections(runtimePlugin)
              .find(
                (connection) =>
                  connection.module instanceof NormalModule &&
                  connection.module.resource.includes('runtime-specific-pkg'),
              )
          : undefined;
        const secondaryState = connection?.getActiveState('runtime-secondary');
        const federationState = connection?.getActiveState(
          'runtime-hoist_inactive_connections',
        );
        const globalState = connection?.getActiveState(undefined);

        if (
          secondaryState !== true ||
          federationState !== false ||
          globalState === false
        ) {
          throw new Error(
            `Expected mixed-runtime connection, got secondary=${String(secondaryState)}, federation=${String(federationState)}, global=${String(globalState)}`,
          );
        }
      });
    });
  }
}

export default defineConfig({
  entry: {
    main: './index.js',
    secondary: './secondary.js',
  },
  target: 'async-node',
  output: {
    filename: '[name].js',
    publicPath: '/',
  },
  optimization: {
    runtimeChunk: {
      name: (entrypoint) => `runtime-${entrypoint.name}`,
    },
    sideEffects: true,
    minimize: false,
    concatenateModules: false,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  plugins: [
    new AssertMixedRuntimeConnectionPlugin(),
    new ModuleFederationPlugin({
      name: 'hoist_inactive_connections',
      filename: 'container.js',
      exposes: {
        './noop': './noop.js',
      },
      runtimePlugins: [path.resolve(import.meta.dirname, 'runtime-plugin.js')],
    }),
  ],
});

import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

const remotes = {
  r07: 'r07@http://localhost:3007/remoteEntry.js',
  r15: 'r15@http://localhost:3015/remoteEntry.js',
  r23: 'r23@http://localhost:3023/remoteEntry.js',
  r31: 'r31@http://localhost:3031/remoteEntry.js',
  r39: 'r39@http://localhost:3039/remoteEntry.js',
  r47: 'r47@http://localhost:3047/remoteEntry.js',
};

function createConfig(name: string, entry: string) {
  return defineConfig({
    mode: 'development',
    devtool: false,
    entry,
    output: {
      filename: `${name}.js`,
      uniqueName: name,
      publicPath: 'auto',
    },
    optimization: {
      chunkIds: 'named',
      moduleIds: 'named',
    },
    plugins: [
      new ModuleFederationPlugin({
        name,
        filename: `${name}-container.js`,
        manifest: false,
        remotes,
      }),
    ],
  });
}

export default defineConfig([
  createConfig('forward', './index-forward.js'),
  createConfig('reverse', './index-reverse.js'),
]);

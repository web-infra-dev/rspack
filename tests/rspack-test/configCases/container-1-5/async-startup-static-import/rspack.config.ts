import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

export default defineConfig(
  ([false, 'single'] as const).map((runtimeChunk, index) => ({
    output: {
      filename: `[name]${index}.js`,
      library: { type: 'commonjs-module' },
    },
    optimization: {
      runtimeChunk,
    },
    plugins: [
      new container.ModuleFederationPlugin({
        name: `async_startup_static_import_${index}`,
        remotes: {
          remote:
            "promise Promise.resolve().then(() => ({ get: () => Promise.resolve(() => 'remote'), init: () => {} }))",
        },
        shared: {
          shared: {
            import: './shared.js',
            version: '1.0.0',
            requiredVersion: '*',
          },
        },
        experiments: { asyncStartup: true },
      }),
    ],
  })),
);

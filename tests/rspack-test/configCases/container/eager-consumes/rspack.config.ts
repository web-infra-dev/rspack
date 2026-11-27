import { defineConfig } from '@rspack/cli';
import { container, sharing } from '@rspack/core';

const { ModuleFederationPluginV1 } = container;
const { ConsumeSharedPlugin, ProvideSharedPlugin } = sharing;
const eager = {
  import: false,
  shareKey: 'lib',
  requiredVersion: '^1.0.0',
  eager: true,
} as const;

export default defineConfig({
  mode: 'development',
  plugins: [
    new ModuleFederationPluginV1({
      name: 'host',
      remotes: {
        remote: `promise Promise.resolve({
          init(scope) {
            scope.lib["1.2.0"] = {
              eager: false,
              from: "remote",
              get: () => Promise.resolve(() => "remote")
            };
          },
          get: () => () => "remote"
        })`,
      },
    }),
    new ProvideSharedPlugin({
      provides: {
        './eager': {
          shareKey: 'lib',
          version: '1.0.0',
          eager: true,
        },
      },
    }),
    new ConsumeSharedPlugin({
      consumes: {
        'eager-sync': { ...eager, strictVersion: true },
        'eager-async': { ...eager, strictVersion: false },
        'non-eager-async': { ...eager, eager: false, strictVersion: false },
      },
    }),
  ],
});

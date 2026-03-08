import path from 'node:path';
import { defineConfig } from '@rsbuild/core';
import { pluginReact } from '@rsbuild/plugin-react';
import { rspack } from '@rspack/core';
import { Layers, pluginRSC } from 'rsbuild-plugin-rsc';

const containerName = 'rsbuild_remote';
type EnvironmentName = 'server' | 'client';

const unifiedExposes = {
  './button': {
    import: './src/exposed-client.tsx',
    environments: ['server', 'client'],
    layers: {
      server: Layers.rsc,
    },
  },
  './composed': {
    import: './src/exposed-composed.tsx',
    environments: ['server', 'client'],
    layers: {
      server: Layers.rsc,
    },
  },
  './consumer': {
    import: './src/rsc-consumer.ts',
    environments: ['server'],
    layers: {
      server: Layers.rsc,
    },
  },
  './server-mixed': {
    import: './src/server-mixed-consumer.ts',
    environments: ['server'],
    layers: {
      server: Layers.rsc,
    },
  },
} as const;

function resolveExposes(environment: EnvironmentName) {
  return Object.fromEntries(
    Object.entries(unifiedExposes)
      .filter(([, definition]) => definition.environments.includes(environment))
      .map(([exposeKey, definition]) => [
        exposeKey,
        {
          import: definition.import,
          ...(definition.layers[environment]
            ? { layer: definition.layers[environment] }
            : {}),
        },
      ]),
  );
}

const rscLayerShared = {
  singleton: true,
  requiredVersion: false,
  shareScope: 'rsc',
  layer: Layers.rsc,
  issuerLayer: Layers.rsc,
} as const;

const ssrLayerShared = {
  singleton: true,
  requiredVersion: false,
  shareScope: 'ssr',
  layer: Layers.ssr,
  issuerLayer: Layers.ssr,
} as const;

const clientLayerShared = {
  singleton: true,
  requiredVersion: false,
  shareScope: 'client',
} as const;

export default defineConfig({
  plugins: [
    pluginReact(),
    pluginRSC({
      layers: {
        ssr: path.join(import.meta.dirname, './src/framework/entry.ssr.tsx'),
      },
    }),
  ],
  environments: {
    server: {
      source: {
        entry: {
          index: {
            import: './src/framework/entry.rsc.tsx',
            layer: Layers.rsc,
          },
        },
      },
    },
    client: {
      source: {
        entry: {
          index: './src/framework/entry.client.tsx',
        },
      },
    },
  },
  tools: {
    rspack: (config, { target }) => {
      const isServerBuild = target === 'node';
      const environment: EnvironmentName = isServerBuild ? 'server' : 'client';
      if (isServerBuild) {
        config.target = 'async-node';
      }

      config.plugins ||= [];
      config.plugins.push(
        new rspack.container.ModuleFederationPlugin({
          name: containerName,
          filename: isServerBuild ? 'remoteEntry.js' : 'remoteEntry.client.js',
          library: isServerBuild
            ? { type: 'commonjs-module' }
            : { type: 'var', name: containerName },
          manifest: isServerBuild ? true : { fileName: 'mf-manifest.client' },
          exposes: resolveExposes(environment),
          experiments: {
            asyncStartup: true,
          },
          shared: isServerBuild
            ? {
                react: rscLayerShared,
                'react/jsx-runtime': rscLayerShared,
                'react-dom': ssrLayerShared,
                'react-dom/server': ssrLayerShared,
                'react-server-dom-rspack/server.node': rscLayerShared,
                'rsbuild-rsc-federation-shared': {
                  request: 'rsbuild-rsc-federation-shared',
                  import: 'rsbuild-rsc-federation-shared',
                  shareKey: 'rsc-shared-key',
                  shareScope: 'rsc',
                  requiredVersion: '^1.0.0',
                  singleton: true,
                  layer: Layers.rsc,
                  issuerLayer: Layers.rsc,
                },
                'rsbuild-rsc-federation-shared/server-actions': {
                  request: 'rsbuild-rsc-federation-shared/server-actions',
                  import: 'rsbuild-rsc-federation-shared/server-actions',
                  shareKey: 'rsc-shared-actions-key',
                  shareScope: 'rsc',
                  requiredVersion: '^1.0.0',
                  singleton: true,
                  layer: Layers.rsc,
                  issuerLayer: Layers.rsc,
                },
              }
            : {
                react: clientLayerShared,
                'react/jsx-runtime': clientLayerShared,
                'react-dom': clientLayerShared,
              },
        }),
      );

      return config;
    },
  },
});

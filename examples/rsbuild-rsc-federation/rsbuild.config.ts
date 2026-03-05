import path from 'node:path';
import { defineConfig } from '@rsbuild/core';
import { pluginReact } from '@rsbuild/plugin-react';
import { rspack } from '@rspack/core';
import { Layers, pluginRSC } from 'rsbuild-plugin-rsc';

const containerName = 'rsbuild_container';

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
          exposes: isServerBuild
            ? {
                './button': {
                  import: './src/exposed-client.tsx',
                  layer: Layers.rsc,
                },
                './consumer': {
                  import: './src/rsc-consumer.ts',
                  layer: Layers.rsc,
                },
              }
            : {
                './button': {
                  import: './src/exposed-client.tsx',
                },
              },
          remotes: {
            remote: 'remote@http://localhost:3002/remoteEntry.js',
          },
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
  output: {
    minify: false,
  },
});

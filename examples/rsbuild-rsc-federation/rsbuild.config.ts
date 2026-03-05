import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from '@rsbuild/core';
import { pluginReact } from '@rsbuild/plugin-react';
import { rspack } from '@rspack/core';
import { Layers, pluginRSC } from 'rsbuild-plugin-rsc';

const rootDir = fileURLToPath(new URL('.', import.meta.url));

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

export default defineConfig({
  plugins: [
    pluginReact(),
    pluginRSC({
      layers: {
        ssr: path.join(rootDir, './src/framework/entry.ssr.tsx'),
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
      config.module ||= {};
      config.module.rules ||= [];
      config.module.rules.push(
        {
          test: /src[\\/]exposed-client\.tsx$/,
          layer: Layers.rsc,
        },
        {
          test: /src[\\/]rsc-consumer\.ts$/,
          layer: Layers.rsc,
        },
        {
          test: /rsbuild-rsc-federation-shared[\\/]index\.js$/,
          layer: Layers.rsc,
        },
        {
          issuerLayer: Layers.rsc,
          resolve: {
            conditionNames: ['react-server', '...'],
          },
        },
      );

      if (target !== 'node') {
        return config;
      }

      config.optimization ||= {};
      config.optimization.moduleIds = 'deterministic';
      config.optimization.chunkIds = 'deterministic';

      config.output ||= {};
      config.output.chunkFilename ||= '[id].js';

      config.plugins ||= [];
      config.plugins.push(
        new rspack.container.ModuleFederationPlugin({
          name: 'rsbuild_container',
          filename: 'remoteEntry.js',
          library: { type: 'commonjs-module' },
          manifest: true,
          exposes: {
            './button': {
              import: './src/exposed-client.tsx',
              layer: Layers.rsc,
            },
            './consumer': {
              import: './src/rsc-consumer.ts',
              layer: Layers.rsc,
            },
          },
          remotes: {
            remote: 'remote@http://localhost:3002/remoteEntry.js',
          },
          shared: {
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

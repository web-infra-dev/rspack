import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

export default defineConfig({
  plugins: [
    new container.ModuleFederationPlugin({
      name: 'host',
      remoteType: 'script',
      remotes: {
        app: [
          'internal ./stub.js',
          'app@http://a.test/remoteEntry.js',
          'app@http://b.test/remoteEntry.js',
        ],
      },
    }),
  ],
});

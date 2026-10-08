import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.less$/,
        use: ({ resource, realResource, resourceQuery, issuer }) => {
          if (
            !resource?.includes('index.less') ||
            !issuer.includes('index.js') ||
            !realResource?.includes('index.less')
          )
            return [];
          if (resourceQuery === '?test')
            return [
              'less-loader',
              fileURLToPath(import.meta.resolve('./loader.mjs')),
            ];
          else return ['less-loader'];
        },
        type: 'css',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
});

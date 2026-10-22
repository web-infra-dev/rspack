import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  node: false,
  externals: [
    {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    function ({ request, dependencyType }, callback) {
      if (/^(\/\/|custom?:\/\/)/.test(request ?? '')) {
        if (dependencyType === 'css-import')
          return callback(undefined, request, 'css-import');
        if (dependencyType === 'url')
          return callback(undefined, request, 'asset-url');
        return callback(undefined, `var '${request}'`);
      }
      return callback();
    },
  ],
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});

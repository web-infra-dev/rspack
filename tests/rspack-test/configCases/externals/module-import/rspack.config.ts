import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: ['web', 'es2020'],
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    module: true,
    filename: '[name].js',
  },
  entry: {
    a: './a',
    main: './index',
  },
  optimization: {
    concatenateModules: true,
  },
  externalsType: 'module-import',
  externals: [
    function ({ request }, callback) {
      if (request === 'external2') {
        return callback(undefined, 'node-commonjs external2');
      }
      callback();
    },
    {
      external0: 'external0',
      external1: 'external1',
      external3: 'external3',
      fs: 'commonjs fs',
      path: 'commonjs path',
    },
  ],
});

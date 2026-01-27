const path = require('path');

/** @type {import('@rspack/cli').Configuration} */
module.exports = {
  mode: 'development', // will be override to "production" by "--mode"
  extends: ['./base.config.js'],
  output: {
    path: path.resolve(__dirname, 'dist'),
  },
  cache: true,
  experiments: {
    cache: {
      type: 'persistent',
    },
  },
  plugins: [
    {
      apply(compiler) {
        const [dep1, dep2] =
          compiler.options.experiments.cache.buildDependencies;
        if (
          dep1 === path.resolve(__dirname, './rspack.config.js') &&
          dep2 === path.resolve(__dirname, './base.config.js')
        ) {
          console.log('===buildDependencies===');
        }
      },
    },
  ],
};

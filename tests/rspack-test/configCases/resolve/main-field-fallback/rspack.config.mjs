import path from 'node:path';

/**
 * @type {import('webpack').Configuration | import('@rspack/cli').Configuration}
 */
export default {
  devtool: false,
  entry: {
    main: {
      import: './index.js',
    },
  },
  resolve: {
    mainFields: ['module', 'main'],
    extensionAlias: {
      '.js': ['.ts', '.js'],
    },
  },
};

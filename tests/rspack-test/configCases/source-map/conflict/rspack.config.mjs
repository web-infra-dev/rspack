/**
 * @type {import('webpack').Configuration | import('@rspack/cli').Configuration}
 */
export default {
  mode: 'development',
  devtool: 'source-map',
  externals: ['source-map'],
  entry: './index.js',
};

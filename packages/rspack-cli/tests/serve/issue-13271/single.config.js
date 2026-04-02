const statePlugin = require('./state-plugin');

module.exports = {
  name: 'disabled',
  mode: 'development',
  entry: './entry-disabled.js',
  devServer: false,
  plugins: [statePlugin('disabled')],
};

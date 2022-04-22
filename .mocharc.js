const path = require('path');

module.exports = {
  require: require.resolve('tsm'),
  extensions: ['.ts', '.tsx', '.js', '.jsx'],
  spec: ['**/*.spec.ts'],
};

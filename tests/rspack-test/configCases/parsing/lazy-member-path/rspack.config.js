const { DefinePlugin } = require('@rspack/core');

module.exports = {
  mode: 'development',
  plugins: [
    new DefinePlugin({
      'LAZY.path.deep.leaf': JSON.stringify('defined'),
      SYNTHETIC:
        '({ deep: { path: { value: 42, read() { return this.value; } } } })',
    }),
  ],
};

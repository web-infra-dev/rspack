module.exports = {
  mode: 'development',
  devtool: false,
  plugins: [
    {
      name: 'test-plugin',
      apply(compiler) {
        throw new Error('error in plugin');
      },
    },
  ],
  devServer: {},
};

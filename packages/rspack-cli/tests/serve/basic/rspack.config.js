module.exports = {
  mode: 'development',
  devtool: false,
  plugins: [
    {
      name: 'test-plugin',
      apply(compiler) {
        compiler.hooks.compilation.tap('compilation', (compilation) => {
          compilation.hooks.processAssets.tap('assets', (assets) => {
            console.log({
              assets,
            });
          });
        });
      },
    },
  ],
  devServer: {},
};

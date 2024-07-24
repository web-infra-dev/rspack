const webpack = require("webpack");

module.exports = {
  watch: true,
  stats: "none",
  mode: "development",
  plugins: [
    {
      apply(compiler) {
        (compiler.webpack ? compiler.hooks.afterDone : compiler.hooks.done).tap(
          "webpack-cli-test",
          () => {
            console.log(`webpack ${webpack.version}`);
          },
        );
      },
    },
  ],
};

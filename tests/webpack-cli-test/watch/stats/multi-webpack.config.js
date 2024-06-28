const webpack = require("webpack");

module.exports = [
  {
    name: "first",
    mode: "development",
    watch: true,
    stats: "none",
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
  },
  {
    name: "two",
    mode: "development",
    watch: true,
    stats: "none",
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
  },
];

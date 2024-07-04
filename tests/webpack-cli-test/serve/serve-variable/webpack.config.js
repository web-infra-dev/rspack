const HAS_WEBPACK_SERVE = process.env.WEBPACK_SERVE || process.env.WEBPACK_DEV_SERVER;

class CustomTestPlugin {
  constructor(isInEnvironment) {
    this.isInEnvironment = isInEnvironment;
  }
  apply(compiler) {
    compiler.hooks.done.tap("testPlugin", () => {
      if (this.isInEnvironment) {
        console.log("PASS");
      } else {
        console.log("FAIL");
      }
    });
  }
}

module.exports = (env) => {
  return {
    mode: "development",
    devtool: false,
    plugins: [new CustomTestPlugin(HAS_WEBPACK_SERVE && env.WEBPACK_SERVE)],
  };
};

const path = require("path");
const fs = require("fs");
module.exports = (_, { testPath }) => ({
  output: {
    path: path.join(testPath, "__[fullhash]__")
  },
  plugins: [{
    apply(compiler) {
      compiler.hooks.done.tap("Test", (stats) => {
        const dirs = fs.readdirSync(testPath);
        expect(dirs).not.toContain("__[fullhash]__");
      });
    }
  }]
});
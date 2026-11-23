const assert = require("node:assert/strict");

module.exports = {
  mode: "production",
  target: "node",
  devtool: false,
  optimization: {
    minimize: false
  },
  output: {
    chunkFilename: "[name].js"
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.emit.tap("LogicalImportBranchGuardTest", compilation => {
          assert.equal(compilation.getAsset("unreachable-and.js"), undefined);
          assert.equal(compilation.getAsset("unreachable-or.js"), undefined);
          assert.equal(
            compilation.getAsset("unreachable-after-effect.js"),
            undefined
          );
          assert.ok(compilation.getAsset("reachable-mixed.js"));
          assert.ok(compilation.getAsset("reachable-mutable.js"));
        });
      }
    }
  ]
};

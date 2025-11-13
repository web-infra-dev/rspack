const path = require('path');
const fs = require('fs');
module.exports = (_, { testPath }) => ({
  output: {
    webassemblyModuleFilename: "[name].wasm",
  },
  module: {
    rules: [
      {
        test: /\.wasm$/,
        type: "webassembly/async"
      }
    ]
  },
  experiments: {
    asyncWebAssembly: true
  },
  plugins: [{
    apply(compiler) {
      compiler.hooks.done.tap("Test", (_) => {
        const main = fs.readFileSync(path.join(testPath, "bundle0.js"), "utf8");
        expect(main).toContain("[name].wasm");
        const assets = fs.readdirSync(path.join(testPath));
        expect(assets).toContain("[name].wasm");
      });
    }
  }]
});
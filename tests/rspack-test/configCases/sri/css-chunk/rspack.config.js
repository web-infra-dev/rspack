const { SubresourceIntegrityPlugin } = require("@rspack/core");
const fs = require("fs");
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = (_, { testPath }) => ({
  target: "web",
  output: {
    crossOriginLoading: "anonymous",
  },
  experiments: {
    css: true
  },
  plugins: [
    new SubresourceIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
    }),
    {
      apply(compiler) {
        compiler.hooks.afterEmit.tap("AfterEmitPlugin", (compilation) => {
          const content = fs.readFileSync(path.resolve(testPath, "bundle0.js"), "utf-8");
          expect(content).toContain("sriHashes");
          expect(content).toContain("sriCssHashes");
        });
      },
    }
  ],
});
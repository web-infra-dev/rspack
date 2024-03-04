const path = require("path");
const assert = require("assert").strict;
const pluginName = "plugin";

class Plugin {
  apply(compiler) {
    compiler.hooks.compilation.tap("Test", compilation => {
      compilation.hooks.processAssets.tap(
        {
          name: "Test",
          stage: -100
        },
        () => {
          compilation.renameAsset("chunk.js", "renamed.js");
        }
      );
    });

  }
}

/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
  context: __dirname,
  output: {
    chunkFilename: "chunk.js"
  },
  plugins: [new Plugin()]
};

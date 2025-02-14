const { RunInPuppeteerPlugin, createIntegrityPlugin, createHtmlPlugin, getDist } = require("../wsi-test-helper");
const { writeFileSync } = require("fs");
const { join } = require("path");

let gotError = false;

module.exports = {
  entry: {
    index: "./index.js",
  },
  output: {
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  plugins: [
    createIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
    }),
    createHtmlPlugin(),
    new RunInPuppeteerPlugin({
      onStart: (stats) => {
        const otherAsset = Object.keys(stats.compilation.assets).find(
          (key) => key !== "index.js" && key.endsWith(".js")
        );
        writeFileSync(join(getDist(__dirname), otherAsset), 'console.log("corrupted");');
      },
      onConsoleError: (msg) => {
        console.log(msg);
        if (
          msg.match(
            /Failed to find a valid digest in the 'integrity' attribute for resource/
          )
        ) {
          gotError = true;
        }
      },
      onDone: () => {
        if (!gotError) {
          throw new Error("No error was raised");
        }
      },
    }),
  ],
};

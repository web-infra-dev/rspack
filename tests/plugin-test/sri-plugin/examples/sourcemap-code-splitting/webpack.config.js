const { createHtmlPlugin, createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const { readFileSync } = require("fs");
const { join } = require("path");
const expect = require("expect");

module.exports = {
  entry: {
    index: "./index.js",
  },
  output: {
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  devtool: "source-map",
  plugins: [
    createIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
      enabled: true,
    }),
    createHtmlPlugin(),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", (stats) => {
          if (stats && stats.hasErrors()) {
            throw new Error(
              stats
                .toJson()
                .errors.map((error) => error.message)
                .join(", ")
            );
          }
          const findAndStripSriHashString = (filePath, pattern, offset) => {
            const fileContent = readFileSync(
              join(getDist(__dirname), filePath),
              "utf-8"
            );
            return fileContent
              .substring(fileContent.indexOf(pattern) + (offset || 0))
              .match(/\{(.*?)\}/)[0]
              .replace(/\\/g, "")
              .replace(/"/g, "");
          };

          const sriHashesInSource = findAndStripSriHashString(
            join(getDist(__dirname), "index.js"),
            "sha256-",
            -10
          );
          const sriHashesInMap = findAndStripSriHashString(
            join(getDist(__dirname), "index.js.map"),
            "__webpack_require__.sriHashes = "
          );
          expect(sriHashesInSource.length).toEqual(sriHashesInMap.length);
        });
      },
    },
  ],
};

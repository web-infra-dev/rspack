const expect = require("expect");
const htmlparser2 = require("htmlparser2");
const { readFileSync } = require("fs");
const { selectAll } = require("css-select");
const { createIntegrityPlugin, createHtmlPlugin, getDist } = require("../wsi-test-helper");
const { join } = require("path");

module.exports = {
  mode: "production",
  entry: "./index.js",
  output: {
    filename: "bundle.js",
    publicPath: "/",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  plugins: [
    createHtmlPlugin(),
    createIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
    }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tapPromise("wsi-test", async (stats) => {
          expect(stats.compilation.warnings.length).toEqual(0);
          const jsIntegrity =
            stats.toJson().assets.find((asset) => asset.name === "bundle.js")
              .integrity || stats.compilation.assets["bundle.js"].integrity;
          expect(jsIntegrity).toMatch(/^sha/);

          const dom = htmlparser2.parseDocument(
            readFileSync(join(getDist(__dirname), "index.html"), "utf-8")
          );

          const scripts = selectAll("script", dom);
          expect(scripts.length).toEqual(1);
          expect(scripts[0].attribs.crossorigin).toEqual("anonymous");
          expect(scripts[0].attribs.integrity).toEqual(jsIntegrity);
        });
      },
    },
  ],
};
